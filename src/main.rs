mod statement;
mod token;
mod tokenizer;
mod parser;

use std::io::{self, Write};
use tokenizer::Tokenizer;
use parser::Parser;

fn main() -> io::Result<()> {
    println!("Welcome to the SQL Parser!");
    println!("Enter SQL queries (each ending with a semicolon) or type 'exit' to quit.");
    println!("For multi-line queries, press Enter after each line.");
    println!("Press Enter twice to force-parse an incomplete query.");
    println!();

    let mut input = String::new();
    let mut current_query = String::new();
    let mut in_multiline = false;
    let mut empty_line_count = 0;
    
    loop {
        if !in_multiline {
            print!("> ");
        } else {
            print!("... ");
        }
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let trimmed_input = input.trim();
        if !in_multiline && trimmed_input.eq_ignore_ascii_case("exit") {
            break;
        }
        
        if trimmed_input.is_empty() {
            empty_line_count += 1;
            if empty_line_count >= 2 && !current_query.is_empty() {
                // Force parse the incomplete query
                println!("Parsing incomplete query...");
                match parse_sql(&current_query) {
                    Ok(statement) => println!("{:#?}\n", statement),
                    Err(e) => {
                        if !current_query.contains(';') {
                            println!("Error: Missing semicolon at the end of the query");
                        }
                        println!("Error: {}", e);
                        println!("Current query: {}\n", current_query);
                    }
                }
                current_query.clear();
                in_multiline = false;
                empty_line_count = 0;
            }
            continue;
        }
        empty_line_count = 0;

        // Add the input to the current query
        if !current_query.is_empty() {
            current_query.push(' ');
        }
        current_query.push_str(trimmed_input);
        
        // If we see a semicolon, try to parse the complete statement
        if trimmed_input.contains(';') {
            in_multiline = false;
            match parse_sql(&current_query) {
                Ok(statement) => println!("{:#?}\n", statement),
                Err(e) => {
                    println!("Error: {}", e);
                    println!("Current query: {}\n", current_query);
                }
            }
            current_query.clear();
        } else {
            // No semicolon yet, continue collecting input
            in_multiline = true;
        }
    }

    println!("Goodbye!");
    Ok(())
}

fn parse_sql(input: &str) -> Result<statement::Statement, String> {
    // Pre-parse validation
    if input.trim().is_empty() {
        return Err("Empty query".to_string());
    }
    
    // Basic SQL validation
    let lowercase_input = input.to_lowercase();
    if lowercase_input.starts_with("select") {
        if !lowercase_input.contains("from") {
            return Err("SELECT statement must contain FROM clause".to_string());
        }
    } else if lowercase_input.starts_with("create table") {
        if lowercase_input.contains("varchar") && !lowercase_input.contains("varchar(") {
            return Err("VARCHAR type must specify length using VARCHAR(n)".to_string());
        }
    }

    let tokenizer = Tokenizer::new(input);
    let mut parser = Parser::new(tokenizer);
    parser.parse_statement()
}
