use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str;
use token::{Literal, Token, TokenType};

#[derive(Debug)]
pub struct UnexpectedCharError {
    line: u32
}

impl fmt::Display for UnexpectedCharError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.description())
    }
}

impl Error for UnexpectedCharError {
    fn description(&self) -> &str {
        "Unexpected character."
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug)]
pub struct UnterminatedStringError {
    line: u32
}

impl fmt::Display for UnterminatedStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.description())
    }
}

impl Error for UnterminatedStringError {
    fn description(&self) -> &str {
        "Unterminated string."
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub struct Scanner {
    source: String,
    source_len: usize,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
    keywords: HashMap<String, TokenType>
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        let tokens: Vec<Token> = vec![];
        let source_len = source.chars().count();
        let keywords = keywords_map();
        Scanner { source, source_len, tokens, keywords, start: 0, current: 0, line: 1 }
    }

    pub fn scan_tokens(&mut self) -> Result<(), Vec<Box<Error>>> {
        let mut errors: Vec<Box<Error>> = vec![];

        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Err(e) => errors.push(e),
                _ => ()
            }
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::from(""),
            None,
            self.line)
        );

        if errors.len() > 0 {
            Err(errors)
        } else {
            println!("{:?}", self.tokens);
            Ok(())
        }
    }

    fn scan_token(&mut self) -> Result<(), Box<Error>> {
        let c = self.advance();
        match c {
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => {
                self.inc_line();
                Ok(())
            },
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let t_type = self.next_match_else('=', TokenType::BangEqual, TokenType::Bang);
                self.add_token(t_type, None)
            },
            '=' => {
                let t_type = self.next_match_else('=', TokenType::EqualEqual, TokenType::Equal);
                self.add_token(t_type, None)
            },
            '<' => {
                let t_type = self.next_match_else('=', TokenType::LessEqual, TokenType::Less);
                self.add_token(t_type, None)
            },
            '>' => {
                let t_type = self.next_match_else('=', TokenType::GreaterEqual, TokenType::Greater);
                self.add_token(t_type, None)
            },
            '/' => {
                if self.next_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(())
                } else {
                    self.add_token(TokenType::Slash, None)
                }
            },
            '"' => self.handle_string_literal(),
            _ => {
                if is_digit(c) {
                    self.handle_number_literal()
                } else if is_alpha(c) {
                    self.handle_identifier()
                } else {
                    Err(Box::new(UnexpectedCharError { line: self.line }))
                }
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) -> Result<(), Box<Error>> {
        let lexeme = substr(&self.source, self.start, self.current);
        self.tokens.push(Token::new(token_type, lexeme, literal, self.line));
        Ok(())
    }

    fn handle_string_literal(&mut self) -> Result<(), Box<Error>> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.inc_line();
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Box::new(UnterminatedStringError { line: self.line }))
        }

        // closing "
        self.advance();

        let value = substr(&self.source, self.start + 1, self.current - 1);

        self.add_token(TokenType::String, Some(Literal::String(value)))
    }

    fn handle_number_literal(&mut self) -> Result<(), Box<Error>> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let val_string = substr(&self.source, self.start, self.current);
        let val_str = &val_string[..];
        let value = val_str.parse::<f64>().unwrap();

        self.add_token(TokenType::Number, Some(Literal::Number(value)))
    }

    fn handle_identifier(&mut self) -> Result<(), Box<Error>> {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = substr(&self.source, self.start, self.current);
        let token_type = match self.keywords.get(&text) {
            Some(keyword) => *keyword,
            None => TokenType::Identifier
        };

        self.add_token(token_type, None)
    }

    fn advance(&mut self) -> char {
        self.inc_current();
        self.char_at(self.current - 1)
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.char_at(self.current)
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source_len {
            '\0'
        } else {
            self.char_at(self.current + 1)
        }
    }

    fn next_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false
        }

        let curr_char = self.char_at(self.current);

        if curr_char != expected {
            false
        } else {
            self.inc_current();
            true
        }
    }

    fn next_match_else(&mut self, expected: char, expected_token: TokenType, else_token: TokenType) -> TokenType {
        if self.is_at_end() {
            return else_token
        }

        let curr_char = self.char_at(self.current);

        if curr_char != expected {
            else_token
        } else {
            self.inc_current();
            expected_token
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_len - 1
    }

    fn char_at(&self, index: usize) -> char {
        self.source.chars().nth(index).unwrap()
    }

    fn inc_current(&mut self) {
        self.current = self.current + 1
    }

    fn inc_line(&mut self) {
        self.line = self.line + 1
    }
}

fn substr(s: &String, start: usize, end: usize) -> String {
    s.chars().skip(start).take(end - start).collect()
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

fn keywords_map() -> HashMap<String, TokenType> {
    let mut keywords = HashMap::new();
    keywords.insert(String::from("and"), TokenType::And);
    keywords.insert(String::from("class"), TokenType::Class);
    keywords.insert(String::from("else"), TokenType::Else);
    keywords.insert(String::from("false"), TokenType::False);
    keywords.insert(String::from("for"), TokenType::For);
    keywords.insert(String::from("fun"), TokenType::Fun);
    keywords.insert(String::from("if"), TokenType::If);
    keywords.insert(String::from("nil"), TokenType::Nil);
    keywords.insert(String::from("or"), TokenType::Or);
    keywords.insert(String::from("print"), TokenType::Print);
    keywords.insert(String::from("return"), TokenType::Return);
    keywords.insert(String::from("super"), TokenType::Super);
    keywords.insert(String::from("this"), TokenType::This);
    keywords.insert(String::from("true"), TokenType::True);
    keywords.insert(String::from("var"), TokenType::Var);
    keywords.insert(String::from("while"), TokenType::While);
    keywords
}
