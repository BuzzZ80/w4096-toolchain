use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::Read;

pub fn get_input() -> Result<(String, String), String> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Interpret arguments
    match args.len() {
        1 => return Err("Expected at least one argument".to_owned()),
        2 => match &args[1][0..=1] {
            "-s" => return get_std(),   // -s indicates that the file comes from stdin
            _ => {}                     // Assume argument is filename and move on
        },
        _ => return Err("Too many arguments provided".to_owned()),
    };

    let content = read_file(&args[1])?;

    Ok((args[1].to_owned(), content))
}

pub fn read_file(filename: &str) -> Result<String, String> {
    let mut content = String::new(); // Create string buffer to hold the contents of the file

    // Ensure file can be opened
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => {
            // If File::open fails, error.
            return Err(format!(
                "File {} couldn't be opened. File::open(...) returned the following error:\n  {}",
                filename, e
            ));
        }
    };

    // Ensure opened file can be read
    if let Err(e) = file.read_to_string(&mut content) {
        // If read_to_string fails, print error and return
        return Err(format!(
            "File {} couldn't be read. file.read_to_string(...) returned the following error:\n  {}",
            filename, e
        ));
    };

    Ok(content)
}

fn get_std() -> Result<(String, String), String> {
    let stdin = io::stdin();
    let mut buf = String::new();
    match stdin.lock().read_to_string(&mut buf) {
        Ok(n) => println!("BASM-PREPROCESSOR: {n} bytes read from stdin."),
        Err(e) => return Err(format!("Couldn't read from stdin, error:\n  {}", e)),
    }
    Ok(("stdin".to_owned(), buf)) // Return read file plus stdin "filename"
}
