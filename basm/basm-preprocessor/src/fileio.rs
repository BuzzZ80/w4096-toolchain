use std::env;
use std::io;
use std::fs::File;
use std::io::prelude::{Read, Write};

pub fn get_input() -> Result<String, String> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Interpret arguments
    match args.len() {
        1 => return Err("Expected at least one argument".to_owned()),
        2 => match &args[1][0..=1] {
            "-s" => return get_std(),   // -s indicates that the file comes from stdin
            _ => {},                    // Assume argument is filename and move on
        }
        _ => return Err("Too many arguments provided".to_owned()),
    };

    let mut program = String::new();    // Create string buffer to hold the contents of the file
    // Ensure file can be opened
    let mut file = match File::create(&args[1]) {
        Ok(file) => file,
        Err(e) => {
            // If File::create fails, error.
            return Err(format!(
                "File {} couldn't be opened. File::create(...) returned the following error:\n  {}",
                args[1], e
            ));
        }
    };

    // Ensure opened file can be read
    if let Err(e) = file.read_to_string(&mut program) {
        // If read_to_string fails, print error and return
        return Err(format!(
            "File {} couldn't be read. file.read_to_string(...) returned the following error:\n  {}",
            args[1], e
        ));
    };

    Ok(program)
}

fn get_std() -> Result<String, String> {
    let stdin = io::stdin();
    let mut buf = String::new();
    match stdin.lock().read_to_string(&mut buf) {
        Ok(n) => println!("BASM-PREPROCESSOR: {n} bytes read from stdin."),
        Err(e) => return Err(format!("Couldn't read from stdin, error:\n  {}", e)),
    }
    Ok(buf)
}