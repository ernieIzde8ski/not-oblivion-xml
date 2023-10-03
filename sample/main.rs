use std::{error::Error, fs::read_to_string};

use not_oblivion_xml::parse_string;

fn main() -> Result<(), Box<dyn Error>> {
    let file_contents = read_to_string("assets/wiki_sample.nox")?;
    let tokens = parse_string(&file_contents)?;
    match cfg!(debug_assertions) {
        true => println!("{tokens:?}"),
        false => {
            for token in tokens {
                print!("{token} ")
            }
            println!();
        }
    };
    Ok(())
}
