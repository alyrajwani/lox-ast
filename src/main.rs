use std::io::{self, BufRead, BufReader, Read, Write, stdout};
use std::fs::File;
use std::env::args;

mod scanner;
mod error; 
mod token_type;
mod token;

use scanner::*;
use error::*;


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

fn run_file(path: &String) -> io::Result<()> {
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

fn run(source: String) -> Result<(), LoxError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    
    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}   

