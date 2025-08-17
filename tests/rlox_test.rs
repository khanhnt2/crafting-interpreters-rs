#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        fs,
        io::{self, BufWriter},
        path::Path,
        rc::Rc,
    };

    use crafting_interpreters::{
        error::RuntimeException, interpreter::Interpreter, parser::Parser, resolver::Resolver,
        scanner::Scanner, token::Token,
    };

    fn run(source: &str, writer: Rc<RefCell<impl io::Write + 'static>>) {
        let scanner = Scanner::new(source);
        let tokens = scanner.into_iter().collect::<Vec<Token>>();
        let mut parser = Parser::new(tokens);
        let statements = match parser.parse() {
            Ok(stmts) => stmts,
            Err(e) => {
                writeln!(writer.borrow_mut(), "{e}").unwrap();
                return;
            }
        };
        let mut interpreter = Interpreter::new(writer.clone());
        let mut resolver = Resolver::new(&mut interpreter);
        if let Err(e) = resolver.resolve_stmts(&statements) {
            writeln!(writer.borrow_mut(), "{e}").unwrap();
            return;
        }
        match interpreter.interpret(&statements) {
            Ok(_) => {}
            Err(e) => match e {
                RuntimeException::Error(runtime_error) => {
                    writeln!(writer.borrow_mut(), "{runtime_error}").unwrap();
                }
                RuntimeException::Return(runtime_return) => {
                    writeln!(writer.borrow_mut(), "{runtime_return}").unwrap();
                }
                RuntimeException::Break | RuntimeException::Continue => todo!("Why hit this?"),
            },
        }
    }

    pub fn run_script_from_file(path: &Path) -> datatest_stable::Result<()> {
        let expected_output = fs::read(path.with_extension("output"))?;
        let script = fs::read_to_string(path)?;
        let buf: Vec<u8> = Vec::new();
        let writer = Rc::new(RefCell::new(BufWriter::new(buf)));
        run(&script, writer.clone());
        assert_eq!(expected_output, writer.borrow().buffer());
        Ok(())
    }
}

datatest_stable::harness! {{test = tests::run_script_from_file, root = "tests/scripts", pattern = r"\.lox$"}}
