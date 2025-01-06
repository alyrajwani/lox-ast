use std::env::args;
use std::io::{self, stdout, BufRead, Write};

mod environment;
mod error;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod callable;
mod native_functions;
mod lox_function;

use error::*;
use interpreter::*;
use parser::*;
use scanner::*;

pub fn main() {
    let args: Vec<String> = args().collect();
    let mut lox = Lox::new();

    match args.len() {
        1 => lox.run_prompt(),
        2 => lox.run_file(&args[1]).expect("Could not run file"),
        _ => {
            println!("Usage: rlox [script]");
            std::process::exit(64);
        }
    }
}

struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, path: &str) -> io::Result<()> {
        let buf = std::fs::read_to_string(path)?;
        if self.run(buf).is_err() {
            std::process::exit(65);
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
                let _ = self.run(line);
            } else {
                break;
            }
            print!("> ");
            let _ = stdout().flush();
        }
    }

    fn run(&mut self, source: String) -> Result<(), LoxResult> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;

        if parser.success() {
            self.interpreter.interpret(&statements);
        }
        Ok(())
    }
}
