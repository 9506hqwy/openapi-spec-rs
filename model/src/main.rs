mod error;
mod gen;

use self::error::Error;
use self::gen::gen;
use openapi_spec_schema::{
    OpenApi, Operation, PartOpenApi, ReferenceOr, RequestBody, Response, Schema, SchemaType,
    SchemaTypes,
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
    )
    .canonicalize()?;

    let output = PathBuf::from(
        args.next()
            .ok_or(Error::arg("Specify a output directory."))?,
    )
    .canonicalize()?;

    let mut scaned_files = vec![];
    let mut schemas = vec![];

    println!("Schema collecting...");
    collect_dir(&root, &root, &mut scaned_files, &mut schemas)?;

    schemas.sort_unstable_by_key(|s| s.r#ref());

    /*
    for schema in &schemas {
        eprintln!("schame: {}", &schema.r#ref());
    }
    */

    println!("Source Code Generating...");
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

fn collect_dir(
    root: &Path,
    dir: &Path,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_file() && path.file_name().unwrap().to_str().unwrap() == "openapi.yaml" {
            collect_schemas(root, &path, scaned_files, schemas)?;
        } else if path.is_dir() {
            collect_dir(root, &path, scaned_files, schemas)?;
        }
    }

    Ok(())
}

fn collect_schemas(
    root: &Path,
    entry_file: &Path,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    let model = read_openapi(entry_file)?;

    if let Some(paths) = model.paths {
        for (path, item) in paths.values {
            if let Some(op) = item.get {
                collect_schema(root, entry_file, &path, "GET", &op, scaned_files, schemas)?;
            }

            if let Some(op) = item.put {
                collect_schema(root, entry_file, &path, "PUT", &op, scaned_files, schemas)?;
            }

            if let Some(op) = item.post {
                collect_schema(root, entry_file, &path, "POST", &op, scaned_files, schemas)?;
            }

            if let Some(op) = item.delete {
                collect_schema(
                    root,
                    entry_file,
                    &path,
                    "DELETE",
                    &op,
                    scaned_files,
                    schemas,
                )?;
            }

            if let Some(op) = item.options {
                collect_schema(
                    root,
                    entry_file,
                    &path,
                    "OPTIONS",
                    &op,
                    scaned_files,
                    schemas,
                )?;
            }

            if let Some(op) = item.head {
                collect_schema(root, entry_file, &path, "HEAD", &op, scaned_files, schemas)?;
            }

            if let Some(op) = item.patch {
                collect_schema(root, entry_file, &path, "PATCH", &op, scaned_files, schemas)?;
            }

            if let Some(op) = item.trace {
                collect_schema(root, entry_file, &path, "TRACE", &op, scaned_files, schemas)?;
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
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(req) = &op.request_body {
        match req {
            ReferenceOr::Value(v) => {
                collect_request_schema(root, entry_file, path, method, v, scaned_files, schemas)?
            }
            ReferenceOr::Ref(r) => {
                collect_reference(root, entry_file, &r.r#ref, true, scaned_files, schemas)?
            }
        }
    }

    if let Some(res) = &op.responses {
        if let Some(default) = &res.r#default {
            match default {
                ReferenceOr::Value(v) => collect_response_schema(
                    root,
                    entry_file,
                    (path, method, &0),
                    v,
                    scaned_files,
                    schemas,
                )?,
                ReferenceOr::Ref(r) => {
                    collect_reference(root, entry_file, &r.r#ref, true, scaned_files, schemas)?
                }
            }
        }

        for (status, item) in &res.statuses.values {
            match item {
                ReferenceOr::Value(v) => collect_response_schema(
                    root,
                    entry_file,
                    (path, method, status),
                    v,
                    scaned_files,
                    schemas,
                )?,
                ReferenceOr::Ref(r) => {
                    collect_reference(root, entry_file, &r.r#ref, true, scaned_files, schemas)?
                }
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
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    for item in req.content.values() {
        if let Some(schema) = &item.schema {
            match schema.r#ref.as_ref() {
                Some(r) => {
                    collect_reference(root, entry_file, r, true, scaned_files, schemas)?;
                }
                _ => {
                    let schema_name =
                        format!("{}-{}-Request", method.to_lowercase(), resource_name(path));
                    collect_anonymous(
                        root,
                        entry_file,
                        schema,
                        &schema_name,
                        true,
                        scaned_files,
                        schemas,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn collect_response_schema(
    root: &Path,
    entry_file: &Path,
    status: (&str, &str, &u16),
    res: &Response,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(contents) = &res.content {
        for item in contents.values() {
            if let Some(schema) = &item.schema {
                match schema.r#ref.as_ref() {
                    Some(r) => {
                        collect_reference(root, entry_file, r, true, scaned_files, schemas)?;
                    }
                    _ => {
                        let schema_name = format!(
                            "{}-{}-{}Response",
                            status.1.to_lowercase(),
                            resource_name(status.0),
                            status.2
                        );
                        collect_anonymous(
                            root,
                            entry_file,
                            schema,
                            &schema_name,
                            true,
                            scaned_files,
                            schemas,
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn collect_reference(
    root: &Path,
    entry_file: &Path,
    r: &str,
    another_file: bool,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    let (url, component) = r.split_once('#').unwrap();

    let file_path = if url.is_empty() {
        PathBuf::from(entry_file)
    } else {
        let path = url.splitn(4, '/').last().unwrap();
        root.join(path).canonicalize()?
    };

    let domain = domain_name(root, &file_path);

    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap();

    let (_, schema_name) = component.rsplit_once('/').unwrap();

    let schema_ref = format!("{domain}/{file_name}#{schema_name}");

    if schemas.iter().any(|i| i.r#ref() == schema_ref) {
        return Ok(());
    }

    for version in all_version(&file_path, another_file)? {
        if !scaned_files.contains(&version) {
            scaned_files.push(version.clone());
            println!("scanning {:?} {}", &version, scaned_files.len());
            match read_part_openapi(&version) {
                Ok(model) => {
                    for (schema_name, schema) in model
                        .components
                        .as_ref()
                        .ok_or(Error::arg(&format!("Not found components {schema_ref}")))?
                        .schemas
                        .as_ref()
                        .ok_or(Error::arg(&format!("Not found schemas {schema_ref}")))?
                    {
                        let file_name = version.file_name().and_then(|n| n.to_str()).unwrap();

                        let item = SchemaItem {
                            domain_name: domain_name(root, &version),
                            schema_file_name: file_name.to_string(),
                            schema_name: schema_name.to_string(),
                            schema: schema.clone(),
                        };

                        schemas.push(item);

                        if let Some(r) = schema.r#ref.as_ref() {
                            collect_reference(root, &version, r, false, scaned_files, schemas)?;
                        }

                        collect_schema_child(
                            root,
                            &version,
                            schema,
                            schema_name,
                            false,
                            scaned_files,
                            schemas,
                        )?;
                    }
                }
                _ => {
                    eprintln!("Failed to read {:?}", &version);
                }
            }
        }
    }

    Ok(())
}

fn collect_schema_child(
    root: &Path,
    entry_file: &Path,
    parent: &Schema,
    parent_name: &str,
    another_file: bool,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    collect_schema_property(
        root,
        entry_file,
        parent,
        parent_name,
        another_file,
        scaned_files,
        schemas,
    )?;

    if let Some(any_of) = parent.any_of.as_ref() {
        for s in any_of {
            if let Some(r) = s.r#ref.as_ref() {
                collect_reference(root, entry_file, r, another_file, scaned_files, schemas)?;
            }
            // TODO: anonymous
        }
    }

    Ok(())
}

fn collect_schema_property(
    root: &Path,
    entry_file: &Path,
    entry: &Schema,
    entry_name: &str,
    another_file: bool,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(properties) = entry.properties.as_ref() {
        for (prop_name, property) in properties {
            if let Some(r) = property.r#ref.as_ref() {
                collect_reference(root, entry_file, r, another_file, scaned_files, schemas)?;
            }

            if let Some(i) = property.items.as_ref() {
                match i.r#ref.as_ref() {
                    Some(r) => {
                        collect_reference(
                            root,
                            entry_file,
                            r,
                            another_file,
                            scaned_files,
                            schemas,
                        )?;
                    }
                    _ => {
                        let schema_name = format!("{}-{}", entry_name, prop_name);
                        collect_anonymous(
                            root,
                            entry_file,
                            i,
                            &schema_name,
                            another_file,
                            scaned_files,
                            schemas,
                        )?;
                    }
                }
            }

            if let Some(any_of) = property.any_of.as_ref() {
                for s in any_of {
                    if let Some(r) = s.r#ref.as_ref() {
                        collect_reference(
                            root,
                            entry_file,
                            r,
                            another_file,
                            scaned_files,
                            schemas,
                        )?;
                    }
                    // TODO: anonymous
                }
            }

            if anonymous_ty(property) {
                let schema_name = format!("{entry_name}-{prop_name}");

                let anonymous = SchemaItem {
                    domain_name: domain_name(root, entry_file),
                    schema_file_name: entry_file
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap()
                        .to_string(),
                    schema: property.clone(),
                    // join '-' because of splitting '_' for name conversion using `Config` struct.
                    schema_name: schema_name.clone(),
                };

                schemas.push(anonymous);

                collect_schema_property(
                    root,
                    entry_file,
                    property,
                    &schema_name,
                    another_file,
                    scaned_files,
                    schemas,
                )?;
            }
        }
    }

    Ok(())
}

fn collect_anonymous(
    root: &Path,
    entry_file: &Path,
    schema: &Schema,
    schema_name: &str,
    another_file: bool,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    schemas.push(SchemaItem {
        domain_name: domain_name(root, entry_file),
        schema_file_name: entry_file
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap()
            .to_string(),
        schema_name: schema_name.to_string(),
        schema: schema.clone(),
    });

    collect_schema_child(
        root,
        entry_file,
        schema,
        schema_name,
        another_file,
        scaned_files,
        schemas,
    )?;

    Ok(())
}

fn anonymous_ty(schema: &Schema) -> bool {
    schema.r#type == Some(SchemaTypes::Unit(SchemaType::Object)) && schema.r#ref.is_none()
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
        let domain_name = &self.domain_name;
        let schema_file_name = &self.schema_file_name;
        let schema_name = &self.schema_name;
        format!("{domain_name}/{schema_file_name}#{schema_name}")
    }
}

// NOTICE: consider name conversion for redfish schema only.

// TODO: missing http://redfish.dmtf.org/schemas/v1/PhysicalContext.v1_0_0.yaml
// TODO: missing http://redfish.dmtf.org/schemas/swordfish/v1/EndpointGroup.yaml

fn all_version(path: &Path, another_file: bool) -> Result<Vec<PathBuf>, Error> {
    let mut versions = vec![PathBuf::from(path).canonicalize()?];

    let file_name = path.file_stem().and_then(|f| f.to_str()).unwrap();
    let (schema, version) = file_name.split_once('.').unwrap_or((file_name, ""));

    if another_file && !version.is_empty() {
        for entry in path.parent().unwrap().read_dir()? {
            let tmp_path = entry?.path();
            if tmp_path.is_file() {
                let tmp_file_name = tmp_path.file_stem().and_then(|f| f.to_str()).unwrap();
                let (tmp_schema, tmp_version) =
                    tmp_file_name.split_once('.').unwrap_or((tmp_file_name, ""));

                if schema == tmp_schema && tmp_version.is_empty() {
                    versions.push(tmp_path.canonicalize()?);
                }
            }
        }
    }

    Ok(versions)
}

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

fn resource_name(source: &str) -> String {
    let mut path = vec![];

    for p in source.split('/').skip(3) {
        if p.contains('{') {
            path.push(p.strip_prefix('{').unwrap().strip_suffix('}').unwrap())
        } else {
            path.push(p);
        }
    }

    path.join("-").to_string()
}
