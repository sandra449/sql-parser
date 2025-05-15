/// Parser module for SQL statements
/// This module implements a Pratt parser for SQL expressions and statements.
/// It handles both SELECT and CREATE TABLE statements with their various clauses.
use crate::statement::{Expression, BinaryOperator, UnaryOperator, Statement, TableColumn, DBType, Constraint};
use crate::token::{Token, Keyword};
use std::iter::Peekable;

/// Parser struct that handles the parsing of SQL statements
/// It uses a peekable iterator of tokens as input and maintains the current token being processed
pub struct Parser<I: Iterator<Item = Result<Token, String>>> {
    tokens: Peekable<I>,
    current_token: Option<Token>,
}

/// Operator precedence levels for the Pratt parser
/// Higher numbers indicate higher precedence
#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    None = 0,
    Or = 1,      // OR operator
    And = 2,     // AND operator
    Equality = 3, // =, != comparisons
    Compare = 4,  // <, >, <=, >= comparisions
    Term = 5,     // +, - arithmetic
    Factor = 6,   // *, / arithmetic
    Unary = 7,    // -, NOT unary operations
    Primary = 8,  // literals, identifiers, parentheses
}

impl<I: Iterator<Item = Result<Token, String>>> Parser<I> {
    /// Creates a new Parser instance with the given token iterator
    pub fn new(tokens: I) -> Self {
        let mut parser = Parser {
            tokens: tokens.peekable(),
            current_token: None,
        };
        parser.advance();
        parser
    }

    fn advance(&mut self) -> Option<Token> {
        self.current_token = self.tokens.next().and_then(|result| result.ok());
        self.current_token.clone()
    }

    fn peek_token(&mut self) -> Option<Token> {
        self.tokens.peek().and_then(|result| result.as_ref().ok().cloned())
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        match self.current_token.clone() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(format!("Expected {:?}, got {:?}", expected, token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn expect_keyword(&mut self, expected: Keyword) -> Result<(), String> {
        match self.current_token.clone() {
            Some(Token::Keyword(keyword)) if keyword == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(format!("Expected keyword {:?}, got {:?}", expected, token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn get_precedence(&self, token: &Token) -> Precedence {
        match token {
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Multiply | Token::Divide => Precedence::Factor,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::GreaterThan | Token::GreaterThanOrEqual |
            Token::LessThan | Token::LessThanOrEqual => Precedence::Compare,
            Token::Keyword(Keyword::And) => Precedence::And,
            Token::Keyword(Keyword::Or) => Precedence::Or,
            _ => Precedence::None,
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token.clone() {
            Some(Token::Keyword(Keyword::Select)) => self.parse_select(),
            Some(Token::Keyword(Keyword::Create)) => self.parse_create_table(),
            Some(token) => Err(format!("Expected SELECT or CREATE, got {:?}", token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.advance(); // Skip SELECT

        // Parse columns
        let mut columns = Vec::new();
        
        // Handle SELECT * case
        if let Some(Token::Multiply) = self.current_token {
            self.advance();
            columns.push(Expression::Identifier("*".to_string()));
        } else {
            // Parse column list
            loop {
                columns.push(self.parse_expression()?);
                
                match self.current_token {
                    Some(Token::Comma) => {
                        self.advance();
                        continue;
                    }
                    Some(Token::Keyword(Keyword::From)) => break,
                    Some(ref token) => return Err(format!("Expected FROM or comma, got {:?}", token)),
                    None => return Err("Unexpected end of input".to_string()),
                }
            }
        }

        // Parse FROM clause
        self.expect_keyword(Keyword::From)?;
        let from = match self.current_token.take() {
            Some(Token::Identifier(table_name)) => {
                self.advance();
                table_name
            }
            Some(token) => return Err(format!("Expected table name, got {:?}", token)),
            None => return Err("Unexpected end of input".to_string()),
        };

        // Parse optional WHERE clause
        let mut where_clause = None;
        if let Some(Token::Keyword(Keyword::Where)) = self.current_token {
            self.advance();
            where_clause = Some(self.parse_expression()?);
        }

        // Parse optional ORDER BY clause
        let mut orderby = Vec::new();
        if let Some(Token::Keyword(Keyword::Order)) = self.current_token {
            self.advance();
            self.expect_keyword(Keyword::By)?;

            loop {
                orderby.push(self.parse_order_by_expr()?);
                
                match self.current_token {
                    Some(Token::Comma) => {
                        self.advance();
                        continue;
                    }
                    Some(Token::Semicolon) | None => break,
                    Some(ref token) => return Err(format!("Expected semicolon or comma, got {:?}", token)),
                }
            }
        }

        // Expect semicolon at the end
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::Select {
            columns,
            from,
            r#where: where_clause,
            orderby,
        })
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.advance(); // Skip CREATE
        self.expect_keyword(Keyword::Table)?;

        // Parse table name
        let table_name = match self.current_token.take() {
            Some(Token::Identifier(name)) => {
                self.advance();
                name
            }
            Some(token) => return Err(format!("Expected table name, got {:?}", token)),
            None => return Err("Unexpected end of input".to_string()),
        };

        // Expect opening parenthesis
        self.expect_token(Token::LeftParentheses)?;

        // Parse column definitions
        let mut column_list = Vec::new();
        loop {
            let column = self.parse_column_definition()?;
            column_list.push(column);

            match self.current_token {
                Some(Token::Comma) => {
                    self.advance();
                    continue;
                }
                Some(Token::RightParentheses) => break,
                Some(ref token) => return Err(format!("Expected comma or closing parenthesis, got {:?}", token)),
                None => return Err("Unexpected end of input".to_string()),
            }
        }

        // Expect closing parenthesis and semicolon
        self.expect_token(Token::RightParentheses)?;
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::CreateTable {
            table_name,
            column_list,
        })
    }

    fn parse_column_definition(&mut self) -> Result<TableColumn, String> {
        // Parse column name
        let column_name = match &self.current_token {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(format!("Expected column name identifier, got {:?}", token)),
            None => return Err("Unexpected end of input while parsing column name".to_string()),
        };

        // Parse column type
        let column_type = match &self.current_token {
            Some(Token::Keyword(Keyword::Int)) => {
                self.advance();
                DBType::Int
            }
            Some(Token::Keyword(Keyword::Bool)) => {
                self.advance();
                DBType::Bool
            }
            Some(Token::Keyword(Keyword::Varchar)) => {
                self.advance();
                self.expect_token(Token::LeftParentheses)
                    .map_err(|_| "Expected '(' after VARCHAR".to_string())?;
                
                let length = match &self.current_token {
                    Some(Token::Number(n)) => {
                        let length = *n as usize;
                        self.advance();
                        length
                    }
                    Some(token) => return Err(format!("Expected number for VARCHAR length, got {:?}", token)),
                    None => return Err("Unexpected end of input while parsing VARCHAR length".to_string()),
                };
                
                self.expect_token(Token::RightParentheses)
                    .map_err(|_| "Expected ')' after VARCHAR length".to_string())?;
                DBType::Varchar(length)
            }
            Some(token) => return Err(format!("Expected column type (INT, BOOL, or VARCHAR), got {:?}", token)),
            None => return Err("Unexpected end of input while parsing column type".to_string()),
        };

        // Parse optional constraints
        let mut constraints = Vec::new();
        loop {
            match &self.current_token {
                Some(Token::Keyword(Keyword::Primary)) => {
                    self.advance();
                    match &self.current_token {
                        Some(Token::Keyword(Keyword::Key)) => {
                            self.advance();
                            constraints.push(Constraint::PrimaryKey);
                        }
                        Some(token) => return Err(format!("Expected KEY after PRIMARY, got {:?}", token)),
                        None => return Err("Unexpected end of input after PRIMARY".to_string()),
                    }
                }
                Some(Token::Keyword(Keyword::Not)) => {
                    self.advance();
                    match &self.current_token {
                        Some(Token::Keyword(Keyword::Null)) => {
                            self.advance();
                            constraints.push(Constraint::NotNull);
                        }
                        Some(token) => return Err(format!("Expected NULL after NOT, got {:?}", token)),
                        None => return Err("Unexpected end of input after NOT".to_string()),
                    }
                }
                Some(Token::Keyword(Keyword::Check)) => {
                    self.advance();
                    match &self.current_token {
                        Some(Token::LeftParentheses) => {
                            self.advance();
                            let expr = self.parse_expression()?;
                            match &self.current_token {
                                Some(Token::RightParentheses) => {
                                    self.advance();
                                    constraints.push(Constraint::Check(expr));
                                }
                                Some(token) => return Err(format!("Expected ')' after CHECK expression, got {:?}", token)),
                                None => return Err("Unexpected end of input in CHECK constraint".to_string()),
                            }
                        }
                        Some(token) => return Err(format!("Expected '(' after CHECK, got {:?}", token)),
                        None => return Err("Unexpected end of input after CHECK".to_string()),
                    }
                }
                _ => break,
            }
        }

        Ok(TableColumn {
            column_name,
            column_type,
            constraints,
        })
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_expression_with_precedence(Precedence::None)
    }

    fn parse_expression_with_precedence(&mut self, precedence: Precedence) -> Result<Expression, String> {
        let mut left = self.parse_prefix()?;

        while let Some(token) = self.current_token.clone() {
            let current_precedence = self.get_precedence(&token);
            if precedence >= current_precedence {
                break;
            }
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression, String> {
        match self.current_token.take() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expression::Number(n))
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expression::String(s))
            }
            Some(Token::Identifier(i)) => {
                self.advance();
                Ok(Expression::Identifier(i))
            }
            Some(Token::Keyword(Keyword::True)) => {
                self.advance();
                Ok(Expression::Bool(true))
            }
            Some(Token::Keyword(Keyword::False)) => {
                self.advance();
                Ok(Expression::Bool(false))
            }
            Some(Token::LeftParentheses) => {
                self.advance();
                let expr = self.parse_expression()?;
                match self.current_token {
                    Some(Token::RightParentheses) => {
                        self.advance();
                        Ok(expr)
                    }
                    Some(ref token) => Err(format!("Expected closing parenthesis, got {:?}", token)),
                    None => Err("Expected closing parenthesis, got end of input".to_string()),
                }
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Minus,
                })
            }
            Some(Token::Plus) => {
                self.advance();
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Plus,
                })
            }
            Some(Token::Keyword(Keyword::Not)) => {
                self.advance();
                let expr = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Not,
                })
            }
            Some(token) => Err(format!("Unexpected token in prefix position: {:?}", token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_infix(&mut self, left: Expression) -> Result<Expression, String> {
        match self.current_token.clone() {
            Some(token) => {
                let precedence = self.get_precedence(&token);
                self.advance();
                let right = self.parse_expression_with_precedence(precedence)?;
                
                let operator = match token {
                    Token::Plus => BinaryOperator::Plus,
                    Token::Minus => BinaryOperator::Minus,
                    Token::Multiply => BinaryOperator::Multiply,
                    Token::Divide => BinaryOperator::Divide,
                    Token::GreaterThan => BinaryOperator::GreaterThan,
                    Token::GreaterThanOrEqual => BinaryOperator::GreaterThanOrEqual,
                    Token::LessThan => BinaryOperator::LessThan,
                    Token::LessThanOrEqual => BinaryOperator::LessThanOrEqual,
                    Token::Equal => BinaryOperator::Equal,
                    Token::NotEqual => BinaryOperator::NotEqual,
                    Token::Keyword(Keyword::And) => BinaryOperator::And,
                    Token::Keyword(Keyword::Or) => BinaryOperator::Or,
                    _ => return Err(format!("Invalid infix operator: {:?}", token)),
                };

                Ok(Expression::BinaryOperation {
                    left_operand: Box::new(left),
                    operator,
                    right_operand: Box::new(right),
                })
            }
            None => Err("Unexpected end of input".to_string()),
        }
    }

    pub fn parse_order_by_expr(&mut self) -> Result<Expression, String> {
        let expr = self.parse_expression()?;
        
        // Check for ASC/DESC
        match self.current_token {
            Some(Token::Keyword(Keyword::Asc)) => {
                self.advance();
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Asc,
                })
            }
            Some(Token::Keyword(Keyword::Desc)) => {
                self.advance();
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Desc,
                })
            }
            _ => Ok(expr), // Default to ASC if no direction specified
        }
    }
}
