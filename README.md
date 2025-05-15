# SQL Parser Implementation

A SQL parser implementation in Rust that supports SELECT and CREATE TABLE statements. This project was developed as part of the Programming Languages course.

## Features

- Tokenizer that handles:
  - SQL keywords (SELECT, CREATE, WHERE, etc.)
  - Identifiers and literals
  - Operators and special characters
  - Numbers and strings
  
- Pratt parser for expressions with proper operator precedence
  
- SQL statement parser supporting:
  - SELECT statements with WHERE and ORDER BY clauses
  - CREATE TABLE statements with column constraints
  - Error handling and informative error messages

## Project Structure

- `src/tokenizer.rs` - Implements the SQL tokenizer
- `src/parser.rs` - Contains the Pratt parser and SQL statement parser
- `src/token.rs` - Defines token types and keywords
- `src/statement.rs` - Defines AST structures for SQL statements

## Usage

To run the SQL parser:

1. Clone the repository
2. Run `cargo build` to build the project
3. Run `cargo run` to start the interactive SQL parser
4. Enter SQL queries ending with semicolons

Example queries:
```sql
SELECT name, age FROM users WHERE age > 18;
CREATE TABLE products (id INT PRIMARY KEY, name VARCHAR(100));
```

## Author

Sandra Sanda <sanda.sandra@student.upt.ro>

## License

This project is part of the Programming Languages course at UPT.