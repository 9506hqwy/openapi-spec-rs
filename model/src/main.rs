mod error;
mod gen;

use self::error::Error;
use self::gen::gen;
use openapi_spec_schema::{
    OpenApi, Operation, PartOpenApi, ReferenceOr, RequestBody, Response, Schema,
};
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Error> {
    let mut args = env::args();

    let root = PathBuf::from(
        args.nth(1)
            .ok_or(Error::arg("Specify a directory contains openapi.yaml."))?,
    );

    let output = PathBuf::from(
        args.next()
            .ok_or(Error::arg("Specify a output directory."))?,
    );

    let mut schemas = vec![];

    collect_dir(&root, &root, &mut schemas)?;

    schemas.sort_unstable_by_key(|s| s.r#ref());

    /*
    for schema in &schemas {
        eprintln!("schame: {}", &schema.r#ref());
    }
    */

    gen(&output, &schemas)?;

    Ok(())
}

fn read_openapi(file: &Path) -> Result<OpenApi, Error> {
    let mut content = String::new();
    File::open(file)?.read_to_string(&mut content)?;

    match file
        .extension()
        .ok_or(Error::arg("Not found extension"))?
        .to_str()
        .unwrap()
    {
        "yml" => from_yml(&content),
        "yaml" => from_yml(&content),
        "json" => from_json(&content),
        _ => panic!("Not supported format."),
    }
}

fn from_json(content: &str) -> Result<OpenApi, Error> {
    Ok(serde_json::from_str::<OpenApi>(content)?)
}

fn from_yml(content: &str) -> Result<OpenApi, Error> {
    Ok(serde_yaml::from_str::<OpenApi>(content)?)
}

fn read_part_openapi(file: &Path) -> Result<PartOpenApi, Error> {
    let mut content = String::new();
    File::open(file)?.read_to_string(&mut content)?;

    match file
        .extension()
        .ok_or(Error::arg("Not found extension"))?
        .to_str()
        .unwrap()
    {
        "yml" => from_part_yml(&content),
        "yaml" => from_part_yml(&content),
        "json" => from_part_json(&content),
        _ => panic!("Not supported format."),
    }
}

fn from_part_json(content: &str) -> Result<PartOpenApi, Error> {
    Ok(serde_json::from_str::<PartOpenApi>(content)?)
}

fn from_part_yml(content: &str) -> Result<PartOpenApi, Error> {
    Ok(serde_yaml::from_str::<PartOpenApi>(content)?)
}

fn collect_dir(root: &Path, dir: &Path, schemas: &mut Vec<SchemaItem>) -> Result<(), Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_file() && path.file_name().unwrap().to_str().unwrap() == "openapi.yaml" {
            collect_schemas(root, &path, schemas)?;
        } else if path.is_dir() {
            collect_dir(root, &path, schemas)?;
        }
    }

    Ok(())
}

fn collect_schemas(
    root: &Path,
    entry_file: &Path,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    let model = read_openapi(entry_file)?;

    if let Some(paths) = model.paths {
        for (path, item) in paths.values {
            if let Some(op) = item.get {
                collect_schema(root, entry_file, &path, "GET", &op, schemas)?;
            }

            if let Some(op) = item.put {
                collect_schema(root, entry_file, &path, "PUT", &op, schemas)?;
            }

            if let Some(op) = item.post {
                collect_schema(root, entry_file, &path, "POST", &op, schemas)?;
            }

            if let Some(op) = item.delete {
                collect_schema(root, entry_file, &path, "DELETE", &op, schemas)?;
            }

            if let Some(op) = item.options {
                collect_schema(root, entry_file, &path, "OPTIONS", &op, schemas)?;
            }

            if let Some(op) = item.head {
                collect_schema(root, entry_file, &path, "HEAD", &op, schemas)?;
            }

            if let Some(op) = item.patch {
                collect_schema(root, entry_file, &path, "PATCH", &op, schemas)?;
            }

            if let Some(op) = item.trace {
                collect_schema(root, entry_file, &path, "TRACE", &op, schemas)?;
            }
        }
    }

    Ok(())
}

fn collect_schema(
    root: &Path,
    entry_file: &Path,
    path: &str,
    method: &str,
    op: &Operation,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(req) = &op.request_body {
        match req {
            ReferenceOr::Value(v) => {
                collect_request_schema(root, entry_file, path, method, v, schemas)?
            }
            ReferenceOr::Ref(r) => collect_reference(root, entry_file, &r.r#ref, schemas)?,
        }
    }

    if let Some(res) = &op.responses {
        if let Some(default) = &res.r#default {
            match default {
                ReferenceOr::Value(v) => {
                    collect_response_schema(root, entry_file, path, method, &0, v, schemas)?
                }
                ReferenceOr::Ref(r) => collect_reference(root, entry_file, &r.r#ref, schemas)?,
            }
        }

        for (status, item) in &res.statuses.values {
            match item {
                ReferenceOr::Value(v) => {
                    collect_response_schema(root, entry_file, path, method, status, v, schemas)?
                }
                ReferenceOr::Ref(r) => collect_reference(root, entry_file, &r.r#ref, schemas)?,
            }
        }
    }

    Ok(())
}

fn collect_request_schema(
    root: &Path,
    entry_file: &Path,
    path: &str,
    method: &str,
    req: &RequestBody,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    for item in req.content.values() {
        if let Some(schema) = &item.schema {
            if schema.r#ref.is_none() {
                eprintln!("Not supported request. {} {}", method, path);
                return Ok(());
            }

            let r = schema.r#ref.as_ref().unwrap();
            collect_reference(root, entry_file, r, schemas)?;
        }
    }

    Ok(())
}

fn collect_response_schema(
    root: &Path,
    entry_file: &Path,
    path: &str,
    method: &str,
    status: &u16,
    res: &Response,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(contents) = &res.content {
        for item in contents.values() {
            if let Some(schema) = &item.schema {
                if schema.r#ref.is_none() {
                    eprintln!("Not supported response. {} {} {}", method, status, path);
                    return Ok(());
                }

                let r = schema.r#ref.as_ref().unwrap();
                collect_reference(root, entry_file, r, schemas)?;
            }
        }
    }

    Ok(())
}

fn collect_reference(
    root: &Path,
    entry_file: &Path,
    r: &str,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    let (url, component) = r.split_once('#').unwrap();

    let file_path = if url.is_empty() {
        PathBuf::from(entry_file)
    } else {
        let path = url.splitn(4, '/').last().unwrap();
        root.join(path)
    };

    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap();

    let (_, schema_name) = component.rsplit_once('/').unwrap();

    let schema_ref = format!("{file_name}#{schema_name}");

    if schemas.iter().any(|i| i.r#ref() == schema_ref) {
        return Ok(());
    }

    match read_part_openapi(&file_path) {
        Ok(model) => {
            let schema = model
                .components
                .as_ref()
                .ok_or(Error::arg(&format!("Not found components {schema_ref}")))?
                .schemas
                .as_ref()
                .ok_or(Error::arg(&format!("Not found schemas {schema_ref}")))?
                .get(schema_name)
                .ok_or(Error::arg(&format!("Not found schema {schema_ref}")))?;

            let item = SchemaItem {
                domain_name: domain_name(root, &file_path),
                schema_file_name: file_name.to_string(),
                schema_name: schema_name.to_string(),
                schema: schema.clone(),
            };

            schemas.push(item);

            collect_schema_property(root, &file_path, schema, schemas)?;

            if let Some(any_of) = schema.any_of.as_ref() {
                for s in any_of {
                    if let Some(r) = s.r#ref.as_ref() {
                        collect_reference(root, entry_file, r, schemas)?;
                    }
                }
            }
        }
        _ => {
            eprintln!("Failed to read {:?}", &file_path);
        }
    }

    Ok(())
}

fn collect_schema_property(
    root: &Path,
    entry_file: &Path,
    entry: &Schema,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(properties) = entry.properties.as_ref() {
        for property in properties.values() {
            if let Some(r) = property.r#ref.as_ref() {
                collect_reference(root, entry_file, r, schemas)?;
            }

            if let Some(i) = property.items.as_ref() {
                if let Some(r) = i.r#ref.as_ref() {
                    collect_reference(root, entry_file, r, schemas)?;
                }
            }

            if let Some(any_of) = property.any_of.as_ref() {
                for s in any_of {
                    if let Some(r) = s.r#ref.as_ref() {
                        collect_reference(root, entry_file, r, schemas)?;
                    }
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------

pub struct SchemaItem {
    domain_name: String,
    schema_file_name: String,
    schema_name: String,
    schema: Schema,
}

impl SchemaItem {
    fn r#ref(&self) -> String {
        let schema_file_name = &self.schema_file_name;
        let schema_name = &self.schema_name;
        format!("{schema_file_name}#{schema_name}")
    }
}

// NOTICE: consider name conversion for redfish schema only.

fn domain_name(root: &Path, file_path: &Path) -> String {
    let rel_path = file_path.strip_prefix(root).unwrap();
    let mut schemas = None;

    for name in rel_path.iter() {
        if schemas.is_some() {
            return if name == "v1" {
                "".to_string()
            } else {
                name.to_str().unwrap().to_string()
            };
        }

        if name == "schemas" {
            schemas = Some(name);
        }
    }

    unreachable!();
}
