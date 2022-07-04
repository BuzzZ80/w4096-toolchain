mod codemap;
mod fileio;
mod lexer;

fn main() {
    let program = match fileio::get_input() {
        Ok(s) => s,
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    };

    let mut lexer = lexer::Lexer::new(program.to_owned());

    println!("{}(EOF)", program);
    println!("{:?}", lexer.tokenize())
}
