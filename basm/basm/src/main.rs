mod codemap;
mod fileio;
mod lexer;
mod parser;

fn main() {
    // Get input data
    let (program, map) = match fileio::get_input() {
        Ok(s) => s,
        Err(e) => {
            println!("\x1b[95mBASM:\x1b[0m {}", e);
            return;
        }
    };

    // Create lexer from input data and convert it into smaller parts for processing
    let tokens = match lexer::Lexer::new(&program).tokenize() {
        Ok(t) => t,
        Err((msg, line)) => {
            if let Some(map) = map {
                let (filename, line) = map.get_from(line);
                println!("\x1b[95mBASM:\x1b[0m Error on line {} of {}:\n  {}", line, filename, msg);
            }
            println!("\x1b[95mBASM:\x1b[0m Error on line {}:\n  {}", line, msg);
            return;
        }
    };

    //for tok in tokens.iter() {
    //    println!("{}", tok);
    //}

    let ast = match parser::Parser::new(tokens).parse() {
        Ok(t) => t,
        Err((msg, line)) => {
            if let Some(map) = map {
                let (filename, line) = map.get_from(line);
                println!("\x1b[95mBASM:\x1b[0m Error on line {} of {}:\n  {}", line, filename, msg);
            }
            println!("\x1b[95mBASM:\x1b[0m Error on line {}:\n  {}", line, msg);
            return;
        }
    };

    for expr in ast {
        println!("{}", expr);
    }
}