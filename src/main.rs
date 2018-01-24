extern crate rlox;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;
use rlox::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
    } else if args.len() == 2 {
        let filename = &args[1];
        run_file(filename);
    } else {
        run_prompt().unwrap_or_else(|err| {
            eprintln!("{}", err);
            String::from("")
        });
    }
}

fn run_file(filename: &str) {
    let mut interpreter = Interpreter::new();
    let mut source = String::new();
    let mut f = File::open(filename).unwrap();
    f.read_to_string(&mut source).unwrap();
    match interpreter.run(source) {
        Err(_) => process::exit(65),
        _ => ()
    }
}

fn run_prompt() -> Result<String, Box<Error>> {
    let mut interpreter = Interpreter::new();
    loop {
        let mut line = String::new();
        print!(">");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line)?;
        match interpreter.run(line) {
            Ok(expr) => println!("{}", expr),
            Err(_) => println!("An error occurred.")
        }
    }
}

