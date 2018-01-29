use std::error::Error;
use std::fmt;
use ast;
use token::{Literal, Token, TokenType};

enum StmtResult<'a> {
    Ok(ast::Stmt<'a>, usize),
    Err(&'a str, usize)
}

enum ExprResult<'a> {
    Ok(ast::Expr<'a>, usize),
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
                println!("{}", error);
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
    Ok(statements)
}

fn declaration(tokens: &Vec<Token>, pos: usize) -> StmtResult {
    if match_type(&tokens[pos], vec![TokenType::Var]) {
        var_declaration(tokens, pos + 1)
    } else {
        statement(tokens, pos)
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
    if match_type(&tokens[pos], vec![TokenType::Print]) {
        print_statement(tokens, pos + 1)
    } else {
        expression_statement(tokens, pos)
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
    match equality(tokens, pos) {
        ExprResult::Ok(expr, pos) => {
            if match_type(&tokens[pos], vec![TokenType::Equal]) {
                let value = assignment(tokens, pos + 1);

                match expr {
                    ast::Expr::Variable(var_expr) => {
                        let name = var_expr.name;
                        match value {
                            ExprResult::Ok(val_expr, pos) => ExprResult::Ok(ast::Expr::assign(name, val_expr), pos),
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
        primary(tokens, pos)
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

