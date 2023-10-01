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

    if let Some(paths) = model.paths {
        let mut routes = vec![];

        for (path, item) in paths.values {
            routes.push(Route {
                path,
                get: item.get.is_some(),
                put: item.put.is_some(),
                post: item.post.is_some(),
                delete: item.delete.is_some(),
                options: item.options.is_some(),
                head: item.head.is_some(),
                patch: item.patch.is_some(),
                trace: item.trace.is_some(),
            });
        }

        routes.sort_unstable_by_key(|r| r.path.clone());

        for route in routes {
            if route.get {
                println!("GET     : {}", route.path);
            }

            if route.put {
                println!("PUT     : {}", route.path);
            }

            if route.post {
                println!("POST    : {}", route.path);
            }

            if route.delete {
                println!("DELETE  : {}", route.path);
            }

            if route.options {
                println!("OPTIONS : {}", route.path);
            }

            if route.head {
                println!("HEAD    : {}", route.path);
            }

            if route.patch {
                println!("PATCH   : {}", route.path);
            }

            if route.trace {
                println!("TRACE   : {}", route.path);
            }
        }
    }

    Ok(())
}

fn from_json(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_json::from_str::<OpenApi>(content)?)
}

fn from_yml(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_yaml::from_str::<OpenApi>(content)?)
}

struct Route {
    path: String,
    get: bool,
    put: bool,
    post: bool,
    delete: bool,
    options: bool,
    head: bool,
    patch: bool,
    trace: bool,
}
