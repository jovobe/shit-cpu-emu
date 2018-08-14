#![feature(rust_2018_preview)]

use std::{
    env,
    error::Error,
    fs,
};

mod diag;
mod span;
mod instr;
mod parse;



fn main() -> Result<(), Box<Error>> {
    // Get CLI argument or print error when no argument was passed
    let path = match env::args().nth(1) {
        Some(s) => s,
        None => {
            println!("<input> argument missing!");
            println!("");
            println!("Usage:");
            println!("  assembler <input>");
            std::process::exit(1);
        }
    };

    // Try to load the file
    let src = fs::read_to_string(path)?;

    // Try to parse the file
    let program = parse::parse(&src).map_err(|_| "failed to parse file")?;

    for line in program.lines {
        println!("{:?}", line);
    }

    Ok(())
}
