use std::error::Error;
use std::fmt;
use ast;
use token::{Literal, Token, TokenType};

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


enum ParseResult<'a> {
    Ok(ast::Expr<'a>, usize),
    Err(&'a str, usize)
}

pub fn parse(tokens: &Vec<Token>) -> Result<ast::Expr, Box<Error>> {
    match expression(tokens, 0) {
        ParseResult::Ok(expr, _) => Ok(expr),
        ParseResult::Err(msg, pos) => {
            let token = &tokens[pos];
            let error = ParseError::new(token.line, token.lexeme.clone(), String::from(msg));
            Err(Box::new(error))
        }
    }
}

fn expression(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    equality(tokens, pos)
}

fn equality(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    match comparison(tokens, pos) {
        ParseResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::BangEqual, TokenType::EqualEqual]) {
                let operator = &tokens[pos];
                match comparison(tokens, pos + 1) {
                    ParseResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ParseResult::Err(msg, pos) => return ParseResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ParseResult::Ok(expr, pos)
        },
        ParseResult::Err(msg, pos) => ParseResult::Err(msg, pos)
    }
}

fn comparison(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    match addition(tokens, pos) {
        ParseResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Greater, TokenType::GreaterEqual,
                                         TokenType::Less, TokenType::LessEqual]) {
                let operator = &tokens[pos];
                match addition(tokens, pos + 1) {
                    ParseResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ParseResult::Err(msg, pos) => return ParseResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ParseResult::Ok(expr, pos)
        },
        ParseResult::Err(msg, pos) => ParseResult::Err(msg, pos)
    }
}

fn addition(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    match multiplication(tokens, pos) {
        ParseResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Plus, TokenType::Minus]) {
                let operator = &tokens[pos];
                match multiplication(tokens, pos + 1) {
                    ParseResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ParseResult::Err(msg, pos) => return ParseResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ParseResult::Ok(expr, pos)
        },
        ParseResult::Err(msg, pos) => ParseResult::Err(msg, pos)
    }
}

fn multiplication(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    match unary(tokens, pos) {
        ParseResult::Ok(mut expr, mut pos) => {
            let mut next_tok = &tokens[pos];
            while match_type(next_tok, vec![TokenType::Star, TokenType::Slash]) {
                let operator = &tokens[pos];
                match unary(tokens, pos + 1) {
                    ParseResult::Ok(right, next_pos) => {
                        pos = next_pos;
                        expr = ast::Expr::binary(expr, operator, right);
                    },
                    ParseResult::Err(msg, pos) => return ParseResult::Err(msg, pos)
                }
                next_tok = &tokens[pos];
            }
            ParseResult::Ok(expr, pos)
        },
        ParseResult::Err(msg, pos) => ParseResult::Err(msg, pos)
    }
}

fn unary(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    let next_tok = &tokens[pos];
    if match_type(next_tok, vec![TokenType::Bang, TokenType::Minus]) {
        let operator = &tokens[pos];
        match unary(tokens, pos + 1) {
            ParseResult::Ok(right, pos) =>
                ParseResult::Ok(ast::Expr::unary(operator, right), pos),
            ParseResult::Err(msg, pos) =>
                ParseResult::Err(msg, pos)
        }
    } else {
        primary(tokens, pos)
    }
}

fn primary(tokens: &Vec<Token>, pos: usize) -> ParseResult {
    let token = &tokens[pos];
    match token.token_type {
        TokenType::False =>
            ParseResult::Ok(ast::Expr::literal(Literal::False), pos + 1),
        TokenType::True =>
            ParseResult::Ok(ast::Expr::literal(Literal::True), pos + 1),
        TokenType::Nil =>
            ParseResult::Ok(ast::Expr::literal(Literal::Nil), pos + 1),
        TokenType::Number | TokenType::String => {
            match token.literal.clone() {
                Some(literal) =>
                    ParseResult::Ok(ast::Expr::literal(literal), pos + 1),
                None =>
                    ParseResult::Err("Expect literal value.", pos)
            }
        },
        TokenType::LeftParen => {
            match expression(tokens, pos + 1) {
                ParseResult::Ok(expr, pos) => {
                    match (&tokens[pos]).token_type {
                        TokenType::RightParen =>
                            ParseResult::Ok(ast::Expr::grouping(expr), pos + 1),
                        _ => ParseResult::Err("Expect ')' after expression.", pos)
                    }
                },
                ParseResult::Err(msg, pos) => ParseResult::Err(msg, pos)
            }
        },
        _ => ParseResult::Err("Expect expression", pos)
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
