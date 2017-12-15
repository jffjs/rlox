use std::error::Error;
use std::fmt;
use ast;
use token::{Literal, Token, TokenType};

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
    current: usize,
    tokens: Vec<Token>
}

impl Parser {
    fn expression(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        self.equality(current)
    }

    fn equality(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.comparison(current);

        while self.match_next(current, vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            current += 1;
            let right = self.comparison(current).unwrap();
            let operator = self.previous(current);
            expr = Ok(ast::Expr::Binary(
                ast::Binary::new(
                    Box::new(expr.unwrap()),
                    operator,
                    Box::new(right)
                )
            ));

        }
        expr
    }

    fn comparison(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.addition(current);

        while self.match_next(current, vec![TokenType::Greater, TokenType::GreaterEqual,
                                   TokenType::Less, TokenType::LessEqual]) {
            current += 1;
            let right = self.addition(current).unwrap();
            let operator = self.previous(current);
            expr = Ok(ast::Expr::Binary(
                ast::Binary::new(
                    Box::new(expr.unwrap()),
                    operator,
                    Box::new(right)
                )
            ));
        }
        expr
    }

    fn addition(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.multiplication(current);

        while self.match_next(current, vec![TokenType::Plus, TokenType::Minus]) {
            current += 1;
            let right = self.multiplication(current).unwrap();
            let operator = self.previous(current);
            expr = Ok(ast::Expr::Binary(
                ast::Binary::new(
                    Box::new(expr.unwrap()),
                    operator,
                    Box::new(right)
                )
            ));
        }
        expr
    }

    fn multiplication(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        let mut expr = self.unary(current);

        while self.match_next(current, vec![TokenType::Star, TokenType::Slash]) {
            current += 1;
            let right = self.unary(current).unwrap();
            let operator = self.previous(current);
            expr = Ok(ast::Expr::Binary(
                ast::Binary::new(
                    Box::new(expr.unwrap()),
                    operator,
                    Box::new(right)
                )
            ));
        }
        expr
    }

    fn unary(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        if self.match_next(current, vec![TokenType::Bang, TokenType::Minus]) {
            current += 1;
            let operator = self.previous(current);
            let right = self.unary(current).unwrap();
            return Ok(ast::Expr::Unary(ast::Unary::new(operator, Box::new(right))));
        }

        self.primary(current)
    }

    fn primary(&self, mut current: usize) -> Result<ast::Expr, Box<Error>> {
        if self.match_next(current, vec![TokenType::False, TokenType::True, TokenType::Nil,
                           TokenType::Number, TokenType::String]) {
            current += 1;
            let tok_literal = self.peek(current);
            let literal = Box::new(tok_literal.literal.clone().unwrap());
            return Ok(ast::Expr::Literal(ast::Literal::new(literal)));
        }

        if self.match_next(current, vec![TokenType::LeftParen]) {
            current += 1;
            let expr = self.expression(current).unwrap();
            return match self.consume(current, TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => Ok(ast::Expr::Grouping(ast::Grouping::new(Box::new(expr)))),
                Err(e) => Err(e)
            };
        }

        let tok = self.peek(current);
        Err(Box::new(ParseError::new(tok.line, String::from("Unexpected parse error."))))
    }

    fn match_next(&self, current: usize, token_types: Vec<TokenType>) -> bool {
        for t_type in token_types {
            if self.check(current, t_type) {
                return true;
            }
        }
        false
    }

    fn check(&self, current: usize, token_type: TokenType) -> bool {
        if self.is_at_end(current) {
            false
        } else {
            self.peek(current).token_type == token_type
        }
    }

    fn consume(&self, current: usize, t_type: TokenType, err: &str) -> Result<usize, Box<Error>> {
        if self.check(current, t_type) {
            Ok(current + 1)
        } else {
            let tok = self.peek(current);
            Err(Box::new(ParseError::new(tok.line, err.to_string())))
        }
    }

    // fn advance(&mut self) -> &Token {
    //     if self.is_at_end() {
    //         self.current += 1;
    //     }
    //     self.previous()
    // }

    fn is_at_end(&self, current: usize) -> bool {
        self.peek(current).token_type == TokenType::Eof
    }

    fn peek(&self, current: usize) -> &Token {
        &self.tokens[current]
    }

    fn previous(&self, current: usize) -> &Token {
        &self.tokens[current - 1]
    }

}
