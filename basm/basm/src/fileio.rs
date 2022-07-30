use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::{Read, /*Write*/};
use crate::codemap::CodeMap;

pub fn get_input() -> Result<(String, Option<CodeMap>), String> {
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

    let asm = get_asm(&args[1])?;
    let map = get_map(&args[1])?;

    Ok((asm, map))
}

pub fn get_asm(filename: &str) -> Result<String, String> {
    get_content(get_file(filename)?)
}

pub fn get_map(filename: &str) -> Result<Option<CodeMap>, String> {
    let filename = format!("{}.map", filename);
    let mut file = match get_file(filename.as_str()) {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };
    let mut data = String::new();
    
    match file.read_to_string(&mut data) {
        Ok(_) => {},
        Err(_) => return Err(format!("Couldn't read {filename}")),
    };
    
    match serde_json::from_str(&data) {
        Ok(map_box) => Ok(map_box),
        Err(e) => Err(format!("Couldn't deserialize map file \"{filename}\". Error: {e}")),
    }
}

fn get_std() -> Result<(String, Option<CodeMap>), String> {
    let stdin = io::stdin();
    let mut data = String::new();
    match stdin.lock().read_to_string(&mut data) {
        Ok(n) => println!("\x1b[95mBASM\x1b[90m: {n} bytes read from stdin."),
        Err(e) => return Err(format!("Couldn't read from stdin, error:\n  {}", e)),
    }
    Ok((data, None)) // Return read file plus no codemap
}

fn get_file(filename: &str) -> Result<File, String> {
    // Ensure file can be opened, and if not, return error
    match File::open(filename) {
        Ok(file) => Ok(file),
        Err(_) => {
            return Err(format!("{} Could not be opened.", filename));
        }
    }
}

fn get_content(mut file: File) -> Result<String, String> {
    let mut data = String::new(); // Create string buffer to hold the contents of the file

    // Ensure opened file can be read, and if not, return error
    if let Err(_) = file.read_to_string(&mut data) {
        return Err("Could not be read.".to_owned());
    };

    Ok(data)
}