use std::{
    error::Error,
    fs::read_to_string,
};

use not_oblivion_xml::{extract_tokens, Maybe};

fn main() -> Result<(), Box<dyn Error>> {
    let string = read_to_string("sample/assets/wiki_sample.nox")?;
    for line in string.split('\n') {
        match extract_tokens(line) {
            Maybe::Ok(line) => println!("{}", line),
            Maybe::Err(msg) => println!("ERROR: {}", msg),
            _ => (),
        };
    }

    Ok(())
}
