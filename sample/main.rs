use std::{error::Error, fs::read_to_string};

use not_oblivion_xml::{extract_tokens, Maybe};

fn main() -> Result<(), Box<dyn Error>> {
    let string = read_to_string("assets/wiki_sample.nox")?;
    for line in string.split('\n') {
        match extract_tokens(line) {
            Maybe::Ok(line) => match cfg!(debug_assertions) {
                true => println!("{:?}", line),
                false => println!("{}", line),
            },
            Maybe::Err(msg) => match cfg!(debug_assertions) {
                true => println!("ERROR: {:?}", msg),
                false => println!("ERROR: {}", msg),
            },
            _ => (),
        };
    }

    Ok(())
}
