extern crate rlox;

use rlox::Repl;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

fn main() {
    // run_file("fib.lox");
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
    } else if args.len() == 2 {
        let filename = &args[1];
        run_file(filename);
    } else {
        run_prompt()
    }
}

fn run_file(filename: &str) {
    // let mut interpreter = Interpreter::new();
    let mut interpreter = Repl::new();
    let mut source = String::new();

    match File::open(filename) {
        Ok(mut f) => match f.read_to_string(&mut source) {
            Ok(_) => match interpreter.run(source) {
                Ok(_) => (),
                Err(_) => process::exit(70),
            },
            Err(e) => {
                eprintln!("{}", e);
                process::exit(65);
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            process::exit(65);
        }
    }
}

fn run_prompt() {
    // let mut interpreter = Interpreter::new();
    let mut interpreter = Repl::new();
    loop {
        let mut line = String::new();
        print!(">");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut line) {
            Ok(_) => match interpreter.run(line) {
                Ok(_) => (),
                Err(_) => (),
            },
            Err(e) => panic!(e),
        }
    }
}
