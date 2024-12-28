// imports
use std::io::{self, BufRead, Write, stdout};
use std::env::args;

mod scanner;
mod error; 
mod token_type;
mod token;
mod parser;
mod expr;
mod ast_printer;

use scanner::*;
use error::*;
use parser::*;
use ast_printer::*;

pub fn main() {
	let args: Vec<String> = args().collect();

	if args.len() > 2 {
		println!("Usage: rlox [script]");
		std::process::exit(64);
	} else if args.len() == 2 {
		run_file(&args[1]).expect("Could not run file");
	} else {
		run_prompt();
	}
}

// run interpreter on a given file
fn run_file(path: &str) -> io::Result<()> {
    let buf = std::fs::read_to_string(path)?;
    match run(buf) {
        Ok(_) => {},
        Err(_) => {
            // Ignore; error was already reported in scan_token
            std::process::exit(65);
        }
    }

    Ok(())
}

// run interpreter on command line prompt
fn run_prompt() {
	let stdin = io::stdin();
    print!("> ");
    let _ = stdout().flush();
	for line in stdin.lock().lines() {
		if let Ok(line) = line {
			if line.is_empty() {
				break;
			}
			match run(line) {
                Ok(_) => {},
                Err(_) => {
                    // Ignore; error was already reported in scan_token
                }
            }
		} else {
			break;
       	}
        print!("> ");
        let _ = stdout().flush();
	}
}

// run a single line
fn run(source: String) -> Result<(), LoxError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        None => {}
        Some(expr) => {
            let printer = AstPrinter {};
            println!("AST Printer: \n{}", printer.print(&expr)?);
        }
    }
    Ok(())   
}   

