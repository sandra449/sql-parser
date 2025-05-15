/// Tokenizer module for SQL statments
/// This module implments a tokenizer that converts SQL input strings into a stream of tokens.
/// It handels SQL keywords, identifyers, literals (numbers and strings), and operaters.
use crate::token::{Token, Keyword};
use std::iter::Peekable;
use std::str::Chars;

/// Tokenizer struct that proceses input text character by character
/// It maintains a peekble iterator over the input characters and tracks the curent position
pub struct Tokenizer<'a> {
    input: Peekable<Chars<'a>>,
    current_position: usize,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new Tokenizer instanse with the given input string
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input: input.chars().peekable(),
            current_position: 0,
        }
    }

    /// Skips whitespaces characters in the input
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.input.next();
            self.current_position += 1;
        }
    }

    /// Reads a number token from the input
    /// Handels both integer and desimal numbers
    fn read_number(&mut self) -> Result<Token, String> {
        let mut number = String::new();
        let mut has_decimal = false;
        
        while let Some(&c) = self.input.peek() {
            if c == '.' && !has_decimal {
                has_decimal = true;
                number.push(c);
                self.input.next();
                self.current_position += 1;
                
                // Must have at least one digit after decimal point
                if let Some(&next_c) = self.input.peek() {
                    if !next_c.is_digit(10) {
                        return Err(format!("Expected digit after decimal point, got '{}'", next_c));
                    }
                } else {
                    return Err("Unexpected end of input after decimal point".to_string());
                }
            } else if c.is_digit(10) {
                number.push(c);
                self.input.next();
                self.current_position += 1;
            } else {
                break;
            }
        }
        
        // If it's a decimal number, convert to equivalent integer
        if has_decimal {
            let parts: Vec<&str> = number.split('.').collect();
            if parts.len() == 2 {
                let whole = parts[0].parse::<u64>()
                    .map_err(|_| format!("Invalid integer part in number: {}", parts[0]))?;
                let decimal = parts[1].parse::<u64>()
                    .map_err(|_| format!("Invalid decimal part in number: {}", parts[1]))?;
                let result = whole * 10 + decimal;
                Ok(Token::Number(result))
            } else {
                Err("Invalid decimal number format".to_string())
            }
        } else {
            number.parse::<u64>()
                .map(Token::Number)
                .map_err(|_| format!("Invalid number: {}", number))
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let mut identifier = String::new();
        while let Some(&c) = self.input.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            identifier.push(c);
            self.input.next();
            self.current_position += 1;
        }

        if identifier.is_empty() {
            return Err("Empty identifier".to_string());
        }

        Ok(match identifier.to_uppercase().as_str() {
            "SELECT" => Token::Keyword(Keyword::Select),
            "CREATE" => Token::Keyword(Keyword::Create),
            "TABLE" => Token::Keyword(Keyword::Table),
            "WHERE" => Token::Keyword(Keyword::Where),
            "ORDER" => Token::Keyword(Keyword::Order),
            "BY" => Token::Keyword(Keyword::By),
            "ASC" => Token::Keyword(Keyword::Asc),
            "DESC" => Token::Keyword(Keyword::Desc),
            "FROM" => Token::Keyword(Keyword::From),
            "AND" => Token::Keyword(Keyword::And),
            "OR" => Token::Keyword(Keyword::Or),
            "NOT" => Token::Keyword(Keyword::Not),
            "TRUE" => Token::Keyword(Keyword::True),
            "FALSE" => Token::Keyword(Keyword::False),
            "PRIMARY" => Token::Keyword(Keyword::Primary),
            "KEY" => Token::Keyword(Keyword::Key),
            "CHECK" => Token::Keyword(Keyword::Check),
            "INT" => Token::Keyword(Keyword::Int),
            "BOOL" => Token::Keyword(Keyword::Bool),
            "VARCHAR" => Token::Keyword(Keyword::Varchar),
            "NULL" => Token::Keyword(Keyword::Null),
            _ => Token::Identifier(identifier),
        })
    }

    fn read_string(&mut self, quote: char) -> Result<Token, String> {
        self.input.next(); // Skip the opening quote
        self.current_position += 1;
        
        let mut string = String::new();
        let mut found_closing_quote = false;
        
        while let Some(c) = self.input.next() {
            self.current_position += 1;
            if c == quote {
                found_closing_quote = true;
                break;
            }
            string.push(c);
        }
        
        if !found_closing_quote {
            return Err(format!("Unterminated string literal starting with {}", quote));
        }
        
        Ok(Token::String(string))
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        match self.input.peek() {
            None => Some(Ok(Token::Eof)),
            Some(&c) => {
                Some(match c {
                    '0'..='9' => self.read_number(),
                    'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),
                    '\'' | '"' => self.read_string(c),
                    '(' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::LeftParentheses)
                    },
                    ')' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::RightParentheses)
                    },
                    ',' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Comma)
                    },
                    ';' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Semicolon)
                    },
                    '*' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Multiply)
                    },
                    '/' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Divide)
                    },
                    '+' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Plus)
                    },
                    '-' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Minus)
                    },
                    '=' => {
                        self.input.next();
                        self.current_position += 1;
                        Ok(Token::Equal)
                    },
                    '>' => {
                        self.input.next();
                        self.current_position += 1;
                        if let Some(&'=') = self.input.peek() {
                            self.input.next();
                            self.current_position += 1;
                            Ok(Token::GreaterThanOrEqual)
                        } else {
                            Ok(Token::GreaterThan)
                        }
                    },
                    '<' => {
                        self.input.next();
                        self.current_position += 1;
                        if let Some(&'=') = self.input.peek() {
                            self.input.next();
                            self.current_position += 1;
                            Ok(Token::LessThanOrEqual)
                        } else {
                            Ok(Token::LessThan)
                        }
                    },
                    '!' => {
                        self.input.next();
                        self.current_position += 1;
                        if let Some(&'=') = self.input.peek() {
                            self.input.next();
                            self.current_position += 1;
                            Ok(Token::NotEqual)
                        } else {
                            Err(format!("Expected '=' after '!', got unexpected character"))
                        }
                    },
                    c => {
                        self.input.next();
                        self.current_position += 1;
                        Err(format!("Unexpected character: '{}'", c))
                    }
                })
            }
        }
    }
}
