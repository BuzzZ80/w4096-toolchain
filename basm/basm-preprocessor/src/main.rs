mod fileio;

fn main() {
    let program = match fileio::get_input() {
        Ok(s) => s,
        Err(e) => {
            println!("BASM-PREPROCESSOR: {}", e);
            return;
        }
    };

    println!("{}(EOF)", program);
}
