use openapi_spec_schema::OpenApi;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let path = PathBuf::from(args.nth(1).ok_or("Specify a file path.")?);

    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let model = match path
        .extension()
        .ok_or("Not found extension")?
        .to_str()
        .unwrap()
    {
        "yml" => from_yml(&content),
        "yaml" => from_yml(&content),
        "json" => from_json(&content),
        _ => panic!("Not supported format."),
    }?;

    dbg!(&model);

    Ok(())
}

fn from_json(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_json::from_str::<OpenApi>(content)?)
}

fn from_yml(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_yaml::from_str::<OpenApi>(content)?)
}
