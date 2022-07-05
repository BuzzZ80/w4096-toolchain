use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::{Read, Write};
use crate::codemap::CodeMap;

const ASM_FILENAME: &str = "out.basm";
const MAP_FILENAME: &str = "out.map";

pub fn get_input() -> Result<(String, String), String> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Interpret arguments
    match args.len() {
        1 => return Err("Expected at least one command line argument".to_owned()),
        2 => match &args[1][0..=1] {
            "-s" => return get_std(), // -s indicates that the file comes from stdin
            _ => {}                   // Assume argument is filename and move on
        },
        _ => return Err("Too many arguments provided".to_owned()),
    };

    let data = read_file(&args[1])?;

    Ok((args[1].to_owned(), data))
}

pub fn read_file(filename: &str) -> Result<String, String> {
    let mut data = String::new(); // Create string buffer to hold the contents of the file

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
    if let Err(e) = file.read_to_string(&mut data) {
        // If read_to_string fails, print error and return
        return Err(format!(
            "File {} couldn't be read. file.read_to_string(...) returned the following error:\n  {}",
            filename, e
        ));
    };

    Ok(data)
}

fn get_std() -> Result<(String, String), String> {
    let stdin = io::stdin();
    let mut data = String::new();
    match stdin.lock().read_to_string(&mut data) {
        Ok(n) => println!("BASM-PREPROCESSOR: {n} bytes read from stdin."),
        Err(e) => return Err(format!("Couldn't read from stdin, error:\n  {}", e)),
    }
    Ok(("stdin".to_owned(), data)) // Return read file plus stdin "filename"
}

pub fn write_asm_file(data: &str) -> Result<(), String> {
    let mut file = match File::create(ASM_FILENAME) {
        Ok(f) => f,
        Err(e) => return Err(format!(
            "{} couldn't be created. File::create(...) returned the following error:\n  {}",
            ASM_FILENAME,
            e,
        )),
    };

    if let Err(e) = file.write_all(data.as_bytes()) {
        return Err(format!(
            "{} couldn't be written to. file.write_all(...) returned the following error:\n  {}",
            MAP_FILENAME,
            e,
        ));
    };

    Ok(())
}

pub fn write_map_file(data: &CodeMap) -> Result<(), String> {
    let mut file = match File::create(MAP_FILENAME) {
        Ok(f) => f,
        Err(e) => return Err(format!(
            "{} couldn't be created. File::create(...) returned the following error:\n  {}",
            ASM_FILENAME,
            e,
        )),
    };

    if let Err(e) = file.write_all(&bincode::serialize(data).unwrap()) {
        return Err(format!(
            "{} couldn't be written to. file.write_all(...) returned the following error:\n  {}",
            MAP_FILENAME,
            e,
        ));
    };

    Ok(())
}
