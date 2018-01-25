extern crate rlox;

use std::env;
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
        run_prompt()
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

fn run_prompt() {
    let mut interpreter = Interpreter::new();
    loop {
        let mut line = String::new();
        print!(">");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut line) {
            Ok(_) => match interpreter.run(line) {
                Ok(expr) => println!("{}", expr),
                Err(_) => ()
            },
            Err(e) => panic!(e)
        }
    }
}

