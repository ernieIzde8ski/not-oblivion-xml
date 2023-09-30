use std::{error::Error, fs::read_to_string};

use not_oblivion_xml::{
    parsing::{ExprLine, LineConversionFailure},
    ErrorEnum, TokenConversionFailure,
};

fn main() -> Result<(), Box<dyn Error>> {
    let string = read_to_string("assets/wiki_sample.nox")?;
    for line in string.split('\n') {
        let line = ExprLine::try_from(line);

        match line {
            Ok(line) => match cfg!(debug_assertions) {
                true => println!("{:?}", line),
                false => println!("{}", line),
            },
            Err(e) => {
                if let LineConversionFailure::TokenFailure(err) = &e {
                    if let TokenConversionFailure::NoTokensPresent = err {
                        continue;
                    }
                };
                match cfg!(debug_assertions) {
                    true => println!("ERROR: {:?}", e),
                    false => println!("ERROR: {}", Box::new(e) as Box<dyn ErrorEnum>),
                };
            }
        }
    }
    Ok(())
}
