mod codemap;
mod fileio;
mod lexer;
mod parser;

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
    let mut lexer = lexer::Lexer::new(&filename, program);

    match lexer.tokenize() {
        Ok(()) => {}
        Err(e) => {
            println!("\x1b[95mBASM-PREPROCESSOR:\x1b[0m {}", e);
            return;
        }
    }

    // Create parser from the output of the lexer, then process the data (resolve consts and includes, etc)
    let mut parser = parser::Parser::new(&filename, lexer.tokens.as_slice());

    match parser.parse() {
        Ok(()) => {}
        Err(e) => {
            println!("\x1b[95mBASM-PREPROCESSOR:\x1b[0m {}", e);
            return;
        }
    }

    // Write output to files
    match fileio::write_asm_file(&parser.output){
        Ok(()) => {}
        Err(e) => {
            println!("\x1b[95mBASM-PREPROCESSOR:\x1b[0m {}", e);
            return;
        }
    }

    match fileio::write_map_file(&parser.map) {
        Ok(()) => {}
        Err(e) => {
            println!("\x1b[95mBASM-PREPROCESSOR:\x1b[0m {}", e);
            return;
        }
    }
}
