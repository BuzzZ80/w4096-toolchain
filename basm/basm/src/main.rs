mod codemap;
mod fileio;
mod lexer;

fn main() {
    // Get input data
    let (filename, program) = match fileio::get_input() {
        Ok(s) => s,
        Err(e) => {
            println!("\x1b[95mBASM-PREPROCESSOR:\x1b[0m {}", e);
            return;
        }
    };

    // Create lexer from input data and convert it into smaller parts for processing
    let tokens = match lexer::Lexer::new(&program).tokenize() {
        Ok(t) => t,
        Err(e) => {
            println!("\x1b[95mBASM-PREPROCESSOR:\x1b[0m {}", e);
            return;
        }
    };

    println!("{:?}", tokens);
}
