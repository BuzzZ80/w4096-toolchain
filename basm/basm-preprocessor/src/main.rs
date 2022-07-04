mod codemap;
mod fileio;
mod lexer;
mod parser;

fn main() {
    let (filename, program) = match fileio::get_input() {
        Ok(s) => s,
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    };

    let mut lexer = lexer::Lexer::new(filename.to_owned(), program);

    match lexer.tokenize() {
        Ok(()) => {}
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    }

    let mut parser = parser::Parser::new(filename.to_owned(), lexer.tokens);

    match parser.parse() {
        Ok(()) => {}
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    }

    println!("{}", parser.output);
    println!("{:#?}", parser.map);
}
