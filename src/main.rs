use std::io::{self, BufRead, Write, stdout};
use std::env::args;

mod scanner;
mod error; 
mod token_type;
mod token;
mod parser;
mod expr;
mod interpreter;

use scanner::*;
use error::*;
use parser::*;
use interpreter::*;

pub fn main() {
	let args: Vec<String> = args().collect();
    let mut lox = Lox::new();

	if args.len() > 2 {
		println!("Usage: rlox [script]");
		std::process::exit(64);
	} else if args.len() == 2 {
		lox.run_file(&args[1]).expect("Could not run file");
	} else {
		lox.run_prompt();
	}
}

struct Lox {
    interpreter: Interpreter
}

impl Lox {
    pub fn new() -> Lox {
        Lox { interpreter: Interpreter {} }
    }
    
    pub fn run_file(&mut self, path: &str) -> io::Result<()> {
        let buf = std::fs::read_to_string(path)?;
        match self.run(buf) {
            Ok(_) => {},
            Err(_) => {
                // Ignore; error was already reported in scan_token
                std::process::exit(65);
            }
        }

        Ok(())
    }

    pub fn run_prompt(&mut self) {
	    let stdin = io::stdin();
        print!("> ");
        let _ = stdout().flush();
	    for line in stdin.lock().lines() {
		    if let Ok(line) = line {
			    if line.is_empty() {
				    break;
			    }
			    match self.run(line) {
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

    fn run(&mut self, source: String) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);

        match parser.parse() {
            None => {}
            Some(expr) => {
                self.interpreter.interpret(&expr);
            }
        }
        Ok(())   
    }
}   

