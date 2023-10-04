use std::{error::Error, fs::read_to_string};

use not_oblivion_xml as nox;

fn main() -> Result<(), Box<dyn Error>> {
    let file_contents = read_to_string("assets/wiki_sample.nox")?;
    let tokens = nox::parse_string(&file_contents)?;
    match cfg!(debug_assertions) {
        true => println!("{tokens:?}"),
        false => nox::render_tokens(tokens, &mut std::io::stdout().lock())?,
    };
    Ok(())
}
