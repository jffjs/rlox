use std::error::Error;
use std::fmt;
use ast;
use token::{Literal, Token, TokenType};

enum StmtResult<'a> {
    Ok(ast::Stmt, usize),

    Err(&'a str, usize)
}

enum ExprResult<'a> {
    Ok(ast::Expr, usize),
    Err(&'a str, usize)
}

pub fn parse(tokens: &Vec<Token>) -> Result<Vec<ast::Stmt>, Vec<Box<Error>>> {
    let mut statements: Vec<ast::Stmt> = vec![];
    let mut errors: Vec<Box<Error>> = vec![];
    let mut pos = 0;
    while tokens[pos].token_type != TokenType::Eof {
        match declaration(tokens, pos) {
            StmtResult::Ok(stmt, next_pos) => {
                statements.push(stmt);
                pos = next_pos;
            },
            StmtResult::Err(msg, mut next_pos) => {
                let token = &tokens[next_pos];
                let error = ParseError::new(token.line, token.lexeme.clone(), String::from(msg));
                errors.push(Box::new(error));

                // fast forward to next statement
                if tokens[next_pos].token_type != TokenType::Eof {
                    next_pos += 1;
                }
                let mut next_tok = &tokens[next_pos];
                while next_tok.token_type != TokenType::Eof {
                    let prev_tok = &tokens[next_pos -1];
                    if prev_tok.token_type == TokenType::Semicolon {
                        break;
                    }

                    match next_tok.token_type {
                        TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For |
                        TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => {
                            break;
                        },
                        _ => {
                            next_pos += 1;
                            next_tok = &tokens[next_pos];
                        }
                    }
                }
                pos = next_pos;
            }
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(statements)
    }
}

fn declaration(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    match tokens[pos].token_type {
        TokenType::Fun => fun_declaration(tokens, pos + 1),
        TokenType::Var => var_declaration(tokens, pos + 1),
        _ => statement(tokens, pos)
    }
}

fn fun_declaration<'a>(tokens: &'a Vec<Token>, mut pos: usize) -> StmtResult<'a> {
    match tokens[pos].token_type {
        TokenType::Identifier => {
            let name = &tokens[pos];
            pos += 1;
            match tokens[pos].token_type {
                TokenType::LeftParen => {
                    pos += 1;
                    let mut params: Vec<Token> = vec![];
                    if !check_token(&tokens[pos], TokenType::RightParen) {
                        loop {
                            if params.len() >= 8 {
                                return StmtResult::Err("Cannot have more than 8 arguments.", pos);
                            }

                            match tokens[pos].token_type {
                                TokenType::Identifier => params.push(tokens[pos].clone()),
                                _ => return StmtResult::Err("Expect parameter name.", pos)
                            };

                            pos += 1;
                            match tokens[pos].token_type {
                                TokenType::Comma => pos = pos + 1,
                                _ => break
                            }
                        }
                    }

                    match tokens[pos].token_type {
                        TokenType::RightParen => match tokens[pos + 1].token_type {
                            TokenType::LeftBrace => match block_statement(tokens, pos + 2) {
                                StmtResult::Ok(body, pos) => StmtResult::Ok(ast::Stmt::function(name, params, body), pos),
                                StmtResult::Err(msg, pos) => StmtResult::Err(msg, pos)
                            },
                            _ => StmtResult::Err("Expect '{{' before function or method body.", pos)
                        },
                        _ => StmtResult::Err("Expect ')' after parameters.", pos)
                    }
                },
                _ => StmtResult::Err("Expect '(' after function or method name.", pos)
            }
        },
        _ => StmtResult::Err("Expect function or method name.", pos)
    }
}

fn var_declaration(tokens: &Vec<Token>, mut pos: usize) -> StmtResult {
    if match_type(&tokens[pos], vec![TokenType::Identifier]) {
        let name = &tokens[pos];
        pos += 1;
        if match_type(&tokens[pos], vec![TokenType::Equal]) {
            match expression(tokens, pos + 1) {
                ExprResult::Ok(initializer, pos) => {
                    if match_type(&tokens[pos], vec![TokenType::Semicolon]) {
                        let stmt = ast::Stmt::var_init(name, initializer);
                        StmtResult::Ok(stmt, pos + 1)
                    } else {
                        StmtResult::Err("Expect ';' after variable declaration.", pos)
                    }
                },
                ExprResult::Err(msg, pos) => StmtResult::Err(msg, pos)
            }
        } else {
            if match_type(&tokens[pos], vec![TokenType::Semicolon]) {
                let stmt = ast::Stmt::var(name);
                StmtResult::Ok(stmt, pos + 1)
            } else {
                StmtResult::Err("Expect ';' after variable declaration.", pos)
            }
        }
    } else {
        StmtResult::Err("Expect variable name.", pos)
    }
}

fn statement(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    match tokens[pos].token_type {
        TokenType::If => if_statement(tokens, pos + 1),
        TokenType::For => for_statement(tokens, pos + 1),
        TokenType::LeftBrace => block_statement(tokens, pos +1),
        TokenType::Print => print_statement(tokens, pos + 1),
        TokenType::While => while_statement(tokens, pos + 1),
        _ => expression_statement(tokens, pos)
    }
}

fn if_statement(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    match tokens[pos].token_type {
        TokenType::LeftParen => match expression(tokens, pos + 1) {
            ExprResult::Ok(condition, pos) => match tokens[pos].token_type {
                TokenType::RightParen => match statement(tokens, pos + 1) {
                    StmtResult::Ok(then_branch, pos) => match tokens[pos].token_type {
                        TokenType::Else => match statement(tokens, pos + 1) {
                            StmtResult::Ok(else_branch, pos) =>
                                StmtResult::Ok(ast::Stmt::if_then_else(condition, then_branch, else_branch), pos),
                            StmtResult::Err(msg, pos) => StmtResult::Err(msg, pos)
                        },
                        _ => StmtResult::Ok(ast::Stmt::if_then(condition, then_branch), pos)
                    },
                    StmtResult::Err(msg, pos) => StmtResult::Err(msg, pos)
                },
                _ => StmtResult::Err("Expect ')' after if condition.", pos)
            },
            ExprResult::Err(msg, pos) => StmtResult::Err(msg, pos)
        },
        _ => StmtResult::Err("Expect '(' after 'if'.", pos)
    }
}

fn for_statement(tokens: &Vec<Token>, mut pos: usize) -> StmtResult {
    let mut initializer = None;
    let mut condition = None;
    let mut increment = None;
    match tokens[pos].token_type {
        TokenType::LeftParen => {
            if let Some(stmt_result) = for_stmt_initializer(tokens, pos + 1) {
                match stmt_result {
                    StmtResult::Ok(init, next_pos) => {
                        initializer = Some(init);
                        pos = next_pos;
                    },
                    StmtResult::Err(msg, pos) => return StmtResult::Err(msg, pos)
                }
            } else {
                pos = pos + 2;
            }

            if let Some(expr_result) = for_stmt_condition(tokens, pos) {
                match expr_result {
                    ExprResult::Ok(cond, next_pos) => {
                        condition = Some(cond);
                        pos = next_pos;
                    },
                    ExprResult::Err(msg, pos) => return StmtResult::Err(msg, pos)
                }
            }

            match tokens[pos].token_type {
                TokenType::Semicolon => pos = pos + 1,
                _ => return StmtResult::Err("Expect ';' after loop condition.", pos)
            }

            if let Some(expr_result) = for_stmt_increment(tokens, pos) {
                match expr_result {
                    ExprResult::Ok(inc, next_pos) => {
                        increment = Some(inc);
                        pos = next_pos;
                    },
                    ExprResult::Err(msg, pos) => return StmtResult::Err(msg, pos)
                }
            }

            match tokens[pos].token_type {
                TokenType::RightParen => pos = pos + 1,
                _ => return StmtResult::Err("Expect ')' after for clauses.", pos)
            }

            match statement(tokens, pos) {
                StmtResult::Ok(body, pos) => {
                    let mut tree = match increment {
                        Some(inc_expr) => ast::Stmt::block(vec![body, ast::Stmt::expr(inc_expr)]),
                        None => body
                    };
                    tree = match condition {
                        Some(cond_expr) => ast::Stmt::while_loop(cond_expr, tree),
                        None => ast::Stmt::while_loop(ast::Expr::literal(Literal::True), tree)
                    };
                    match initializer {
                        Some(init_stmt) => StmtResult::Ok(ast::Stmt::block(vec![init_stmt, tree]), pos),
                        None => StmtResult::Ok(tree, pos)
                    }

                },
                StmtResult::Err(msg, pos) => StmtResult::Err(msg, pos)
            }
        },
        _ => StmtResult::Err("Expect '(' after 'for'.", pos)
    }
}

fn for_stmt_initializer(tokens: &Vec<Token>, pos: usize) -> Option<StmtResult> {
    match tokens[pos].token_type {
        TokenType::Semicolon => None,
        TokenType::Var => Some(var_declaration(tokens, pos + 1)),
        _ => Some(expression_statement(tokens, pos + 1))
    }
}

fn for_stmt_condition(tokens: &Vec<Token>, pos: usize) -> Option<ExprResult> {
    match tokens[pos].token_type {
        TokenType::Semicolon => None,
        _ => Some(expression(tokens, pos))
    }
}

fn for_stmt_increment(tokens: &Vec<Token>, pos: usize) -> Option<ExprResult> {
    match tokens[pos].token_type {
        TokenType::RightParen => None,
        _ => Some(expression(tokens, pos))
    }
}

fn while_statement(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    match tokens[pos].token_type {
        TokenType::LeftParen => match expression(tokens, pos + 1) {
            ExprResult::Ok(condition, pos) => match tokens[pos].token_type {
                TokenType::RightParen => match statement(tokens, pos + 1) {
                    StmtResult::Ok(body, pos) => StmtResult::Ok(ast::Stmt::while_loop(condition, body), pos),
                    StmtResult::Err(msg, pos) => StmtResult::Err(msg, pos)
                },
                _ => StmtResult::Err("Expect ')' after if condition.", pos)
            },
            ExprResult::Err(msg, pos) => StmtResult::Err(msg, pos)
        },
        _ => StmtResult::Err("Expect '(' after 'while'.", pos)
    }
}

fn print_statement(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    match expression(tokens, pos) {
        ExprResult::Ok(expr, pos) => {
            let next_tok = &tokens[pos];
            if match_type(next_tok, vec![TokenType::Semicolon]) {
                let stmt = ast::Stmt::print(expr);
                StmtResult::Ok(stmt, pos + 1)
            } else {
                StmtResult::Err("Expect ';' after value.", pos)
            }
        },
        ExprResult::Err(err, pos) => StmtResult::Err(err, pos)
    }
}

fn block_statement(tokens: &Vec<Token>, mut pos: usize) -> StmtResult {
    let mut statements: Vec<ast::Stmt> = vec![];

    let mut next_tok = &tokens[pos];
    while !match_type(next_tok, vec![TokenType::RightBrace]) && next_tok.token_type != TokenType::Eof {
        match declaration(tokens, pos) {
            StmtResult::Ok(statement, next_pos) =>{
                statements.push(statement);
                pos = next_pos;
            },
            StmtResult::Err(msg, next_pos) => return StmtResult::Err(msg, next_pos)
        }
        next_tok = &tokens[pos];
    }

    // consume right brace
    match next_tok.token_type {
        TokenType::RightBrace => StmtResult::Ok(ast::Stmt::block(statements), pos + 1),
        _ => StmtResult::Err("Expect '}' after block.", pos)
    }
}

fn expression_statement(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    match expression(tokens, pos) {
        ExprResult::Ok(expr, pos) => {
            let next_tok = &tokens[pos];
            if match_type(next_tok, vec![TokenType::Semicolon]) {
                let stmt = ast::Stmt::expr(expr);
                StmtResult::Ok(stmt, pos + 1)
            } else {
                StmtResult::Err("Expect ';' after value.", pos)
            }
        },
        ExprResult::Err(err, pos) => StmtResult::Err(err, pos)
    }
}

fn expression(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    assignment(tokens, pos)
}

fn assignment(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match or(tokens, pos) {
        ExprResult::Ok(expr, pos) => {
            if match_type(&tokens[pos], vec![TokenType::Equal]) {
                let value = assignment(tokens, pos + 1);

                match expr {
                    ast::Expr::Variable(var_expr) => {
                        let name = var_expr.name;
                        match value {
                            ExprResult::Ok(val_expr, pos) => ExprResult::Ok(ast::Expr::assign(&name, val_expr), pos),
                            ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
                        }
                    },
                    _ => ExprResult::Err("Invalid assignment target.", pos)
                }
            } else {
                ExprResult::Ok(expr, pos)
            }
        },
        ExprResult::Err(err, pos) => ExprResult::Err(err, pos)
    }
}

fn or(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match and(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Or]) {
                let operator = &tokens[pos];
                match and(tokens, pos + 1) {
                    ExprResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::logical(expr, operator, right);
                    },
                    ExprResult::Err(msg, pos) => return ExprResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(err, pos) => ExprResult::Err(err, pos)
    }
}

fn and(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match equality(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::And]) {
                let operator = &tokens[pos];
                match equality(tokens, pos + 1) {
                    ExprResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::logical(expr, operator, right);
                    },
                    ExprResult::Err(msg, pos) => return ExprResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(err, pos) => ExprResult::Err(err, pos)
    }
}

fn equality(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match comparison(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::BangEqual, TokenType::EqualEqual]) {
                let operator = &tokens[pos];
                match comparison(tokens, pos + 1) {
                    ExprResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ExprResult::Err(msg, pos) => return ExprResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
    }
}

fn comparison(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match addition(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Greater, TokenType::GreaterEqual,
                                         TokenType::Less, TokenType::LessEqual]) {
                let operator = &tokens[pos];
                match addition(tokens, pos + 1) {
                    ExprResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ExprResult::Err(msg, pos) => return ExprResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
    }
}

fn addition(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match multiplication(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Plus, TokenType::Minus]) {
                let operator = &tokens[pos];
                match multiplication(tokens, pos + 1) {
                    ExprResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ExprResult::Err(msg, pos) => return ExprResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
    }
}

fn multiplication(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match unary(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Star, TokenType::Slash]) {
                let operator = &tokens[pos];
                match unary(tokens, pos + 1) {
                    ExprResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ExprResult::Err(msg, pos) => return ExprResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
    }
}

fn unary(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    let next_tok = &tokens[pos];
    if match_type(next_tok, vec![TokenType::Bang, TokenType::Minus]) {
        let operator = &tokens[pos];
        match unary(tokens, pos + 1) {
            ExprResult::Ok(right, pos) =>
                ExprResult::Ok(ast::Expr::unary(operator, right), pos),
            ExprResult::Err(msg, pos) =>
                ExprResult::Err(msg, pos)
        }
    } else {
        call(tokens, pos)
    }
}

fn call(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    match primary(tokens, pos) {
        ExprResult::Ok(mut expr, mut pos) => {
            loop {
                match tokens[pos].token_type {
                    TokenType::LeftParen => {
                        let mut args: Vec<ast::Expr> = vec![];
                        pos += 1;
                        if !check_token(&tokens[pos], TokenType::RightParen) {
                            loop {
                                if args.len() >= 8 {
                                    return ExprResult::Err("Cannot have more than 8 arguments.", pos);
                                }
                                match expression(tokens, pos) {
                                    ExprResult::Ok(arg, next_pos) => {
                                        args.push(arg);
                                        pos = next_pos;
                                    },
                                    ExprResult::Err(msg, pos) =>
                                        return ExprResult::Err(msg, pos)
                                }

                                match tokens[pos].token_type {
                                    TokenType::Comma => pos += 1,
                                    _ => break
                                }
                            }
                        }

                        let paren;
                        match tokens[pos].token_type {
                            TokenType::RightParen => {
                                paren = &tokens[pos];
                                pos += 1;
                            }
                            _ => return ExprResult::Err("Expect ')' after arguments.", pos)
                        }

                        expr = ast::Expr::call(expr, paren, args);
                    },
                    _ => break
                }
            }
            ExprResult::Ok(expr, pos)
        },
        ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
    }
}

fn primary(tokens: &Vec<Token>, pos: usize) -> ExprResult {
    let token = &tokens[pos];
    match token.token_type {
        TokenType::False =>
            ExprResult::Ok(ast::Expr::literal(Literal::False), pos + 1),
        TokenType::True =>
            ExprResult::Ok(ast::Expr::literal(Literal::True), pos + 1),
        TokenType::Nil =>
            ExprResult::Ok(ast::Expr::literal(Literal::Nil), pos + 1),
        TokenType::Number | TokenType::String => {
            match token.literal.clone() {
                Some(literal) =>
                    ExprResult::Ok(ast::Expr::literal(literal), pos + 1),
                None =>
                    ExprResult::Err("Expect literal value.", pos)
            }
        },
        TokenType::LeftParen => {
            match expression(tokens, pos + 1) {
                ExprResult::Ok(expr, pos) => {
                    match (&tokens[pos]).token_type {
                        TokenType::RightParen =>
                            ExprResult::Ok(ast::Expr::grouping(expr), pos + 1),
                        _ => ExprResult::Err("Expect ')' after expression.", pos)
                    }
                },
                ExprResult::Err(msg, pos) => ExprResult::Err(msg, pos)
            }
        },
        TokenType::Identifier => {
            ExprResult::Ok(ast::Expr::variable(token), pos + 1)
        },
        _ => ExprResult::Err("Expect expression", pos)
    }
}

fn match_type(token: &Token, tok_types: Vec<TokenType>) -> bool {
    for tok_type in tok_types {
        if token.token_type == TokenType::Eof {
            return false;
        } else if token.token_type == tok_type {
            return true;
        }
    }
    false
}

fn check_token(token: &Token, tok_type: TokenType) -> bool {
    if token.token_type == TokenType::Eof {
        false
    } else {
        token.token_type == tok_type
    }
}

#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: u32,
    lexeme: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.description())
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl ParseError {
    fn new(line: u32, lexeme: String, msg: String) -> ParseError {
        ParseError { line, lexeme, msg }
    }
}

