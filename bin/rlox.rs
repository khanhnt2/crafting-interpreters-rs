use std::{
    cell::RefCell,
    fs::{self},
    io::{self, Write},
    rc::Rc,
};

use clap::Parser as ClapParser;
use crafting_interpreters::{
    error::RuntimeException, interpreter::Interpreter, parser::Parser, resolver::Resolver,
    scanner::Scanner, token::Token,
};

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file_path: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(file_path) = args.file_path {
        run_file(&file_path);
    } else {
        run_prompt();
    }
}

fn run_file(path: &str) {
    let writer = Rc::new(RefCell::new(io::stdout()));
    let mut interpreter = Interpreter::new(writer);
    let source = fs::read_to_string(path).expect("Failed to read file");
    run(&source, &mut interpreter);
}

fn run_prompt() {
    let writer = Rc::new(RefCell::new(io::stdout()));
    let mut interpreter = Interpreter::new(writer.clone());
    let mut resolver = Resolver::new(&mut interpreter);
    loop {
        write!(writer.borrow_mut(), "> ").unwrap();
        std::io::stdout().flush().expect("Failed to flush stdout");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let scanner = Scanner::new(&input);
        let tokens: Vec<Token> = scanner.into_iter().collect();
        let mut parser = Parser::new(tokens);
        let statements = match parser.parse() {
            Ok(stmts) => stmts,
            Err(e) => {
                writeln!(writer.borrow_mut(), "{e}").unwrap();
                continue;
            }
        };
        if let Err(e) = resolver.resolve_stmts(&statements) {
            writeln!(writer.borrow_mut(), "{e}").unwrap();
            continue;
        }
        if let Err(e) = resolver.interpreter.interpret(&statements) {
            writeln!(writer.borrow_mut(), "{e}").unwrap();
            continue;
        }
    }
}

fn run(source: &str, interpreter: &mut Interpreter) {
    let scanner = Scanner::new(source);
    let tokens = scanner.into_iter().collect::<Vec<Token>>();
    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(stmts) => stmts,
        Err(e) => {
            writeln!(interpreter.writer.borrow_mut(), "{e}").unwrap();
            return;
        }
    };
    let mut resolver = Resolver::new(interpreter);
    if let Err(e) = resolver.resolve_stmts(&statements) {
        writeln!(interpreter.writer.borrow_mut(), "{e}").unwrap();
        return;
    }
    match interpreter.interpret(&statements) {
        Ok(_) => {}
        Err(e) => match e {
            RuntimeException::Error(runtime_error) => {
                writeln!(interpreter.writer.borrow_mut(), "{runtime_error}").unwrap();
            }
            RuntimeException::Return(runtime_return) => {
                writeln!(interpreter.writer.borrow_mut(), "{runtime_return}").unwrap();
            }
            RuntimeException::Break | RuntimeException::Continue => todo!("Why hit this?"),
        },
    }
}
