mod codemap;
mod fileio;
mod lexer;
mod parser;

fn main() {
    let program = match fileio::get_input() {
        Ok(s) => s,
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    };

    let mut lexer = lexer::Lexer::new(program);

    match lexer.tokenize() {
        Ok(()) => {}
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    }

    let mut parser = parser::Parser::new(lexer.tokens);

    match parser.parse() {
        Ok(()) => {}
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    }

    println!("{}", parser.output);
}
