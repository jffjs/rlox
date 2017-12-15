use std::error::Error;
use std::fmt;
use ast;
use token::{Token, TokenType};

#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: u32
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
    fn new(line: u32, msg: String) -> ParseError {
        ParseError { line, msg }
    }
}

pub struct Parser {
    tokens: Vec<Token>
}

impl Parser {
    fn expression(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        self.equality(idx)
    }

    fn equality(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.comparison(idx);

        loop {
            let (idx, matched) = self.match_next(idx, vec![TokenType::BangEqual, TokenType::EqualEqual]);
            if matched {
                let right = self.comparison(idx).unwrap();
                let operator = self.previous(idx);
                expr = Ok(ast::Expr::binary(expr.unwrap(), operator, right));
            } else {
                break;
            }
        }
        expr
    }

    fn comparison(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.addition(idx);

        loop {
            let (idx, matched) = self.match_next(idx, vec![TokenType::Greater, TokenType::GreaterEqual,
                                                            TokenType::Less, TokenType::LessEqual]);
            if matched {
                let right = self.addition(idx).unwrap();
                let operator = self.previous(idx);
                expr = Ok(ast::Expr::binary(expr.unwrap(), operator, right));
            } else {
                break;
            }
        }
        expr
    }

    fn addition(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.multiplication(idx);

        loop {
            let (idx, matched) = self.match_next(idx, vec![TokenType::Plus, TokenType::Minus]);
            if matched {
                let right = self.multiplication(idx).unwrap();
                let operator = self.previous(idx);
                expr = Ok(ast::Expr::binary(expr.unwrap(), operator, right));
            } else {
                break;
            }
        }
        expr
    }

    fn multiplication(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.unary(idx);

        loop {
            let (idx, matched) = self.match_next(idx, vec![TokenType::Star, TokenType::Slash]);
            if matched {
                let right = self.unary(idx).unwrap();
                let operator = self.previous(idx);
                expr = Ok(ast::Expr::binary(expr.unwrap(), operator, right));
            } else {
                break;
            }
        }
        expr
    }

    fn unary(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        let (idx, matched) = self.match_next(idx, vec![TokenType::Bang, TokenType::Minus]);
        if matched {
            let operator = self.previous(idx);
            let right = self.unary(idx).unwrap();
            return Ok(ast::Expr::unary(operator, right));
        }
        self.primary(idx)
    }

    fn primary(&self, idx: usize) -> Result<ast::Expr, Box<Error>> {
        let (idx, lit_match) = self.match_next(idx, vec![TokenType::False, TokenType::True, TokenType::Nil,
                                                          TokenType::Number, TokenType::String]);
        if lit_match {
            let tok_literal = self.peek(idx);
            let literal = tok_literal.literal.clone().unwrap();
            return Ok(ast::Expr::literal(literal));
        }

        let (idx, grouping_match) = self.match_next(idx, vec![TokenType::LeftParen]);
        if grouping_match {
            let expr = self.expression(idx).unwrap();
            return match self.consume(idx, TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => Ok(ast::Expr::grouping(expr)),
                Err(e) => Err(e)
            };
        }

        let tok = self.peek(idx);
        Err(Box::new(ParseError::new(tok.line, String::from("Unexpected parse error."))))
    }

    fn match_next(&self, idx: usize, token_types: Vec<TokenType>) -> (usize, bool) {
        for t_type in token_types {
            if self.check(idx, t_type) {
                return (idx + 1, true);
            }
        }
        (idx, false)
    }

    fn check(&self, idx: usize, token_type: TokenType) -> bool {
        if self.is_at_end(idx) {
            false
        } else {
            self.peek(idx).token_type == token_type
        }
    }

    fn consume(&self, idx: usize, t_type: TokenType, err: &str) -> Result<usize, Box<Error>> {
        if self.check(idx, t_type) {
            Ok(idx + 1)
        } else {
            let tok = self.peek(idx);
            Err(Box::new(ParseError::new(tok.line, err.to_string())))
        }
    }

    // fn advance(&mut self) -> &Token {
    //     if self.is_at_end() {
    //         self.idx += 1;
    //     }
    //     self.previous()
    // }

    fn is_at_end(&self, idx: usize) -> bool {
        self.peek(idx).token_type == TokenType::Eof
    }

    fn peek(&self, idx: usize) -> &Token {
        &self.tokens[idx]
    }

    fn previous(&self, idx: usize) -> &Token {
        &self.tokens[idx - 1]
    }

}
