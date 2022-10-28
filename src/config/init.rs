use std::{fs::read_to_string, io::Error};

lazy_static! {
    static ref PATH: String = String::from("../../reference.txt");
}

pub fn read_allowed_methods() -> Result<Vec<String>, Error> {
    let path = PATH.to_string();

    let content: String = read_to_string(path)?;
    let split = content.split('\n');
    let methods: Vec<String> = split.map(|str| str.to_owned()).collect();

    Ok(methods)
}
