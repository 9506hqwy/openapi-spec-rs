use openapi_spec_schema::model::Any;
use openapi_spec_schema::{
    Components, MediaType, OpenApi, PartOpenApi, ReferenceOr, Response, Schema, SchemaType,
    SchemaTypes,
};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let path = PathBuf::from(args.nth(1).ok_or("Specify a file path.")?);
    let http_method = args.next().ok_or("Specify a HTTP method.")?;
    let http_path = args.next().ok_or("Specify a HTTP path.")?;
    let http_code = args
        .next()
        .map(|c| c.parse::<u16>().expect("Specify number"));

    let model = read_openapi(&path)?;

    let paths = model.paths.as_ref().ok_or("Not exists paths")?;
    let path_object = paths
        .values
        .get(&http_path)
        .ok_or(format!("Not found path {http_path}"))?;

    let method_object = match http_method.to_lowercase().as_str() {
        "get" => Ok(&path_object.get),
        "put" => Ok(&path_object.put),
        "post" => Ok(&path_object.post),
        "delete" => Ok(&path_object.delete),
        "options" => Ok(&path_object.options),
        "head" => Ok(&path_object.head),
        "patch" => Ok(&path_object.patch),
        "trace" => Ok(&path_object.trace),
        _ => Err(format!("Not found method {http_method}")),
    }?;

    if let Some(op) = method_object {
        if let Some(res) = &op.responses {
            if let Some(code) = http_code {
                let body = res
                    .statuses
                    .values
                    .get(&code)
                    .ok_or("Not found status code")?;
                match body {
                    ReferenceOr::Value(v) => dump_response_body((&path, &model.components), v)?,
                    ReferenceOr::Ref(r) => {
                        let v = get_response_body((&path, &model.components), &r.r#ref)?;
                        dump_response_body((&path, &Some(v.0)), &v.1)?
                    }
                }
            } else {
                for code in res.statuses.values.keys() {
                    println!("{code}");
                }
            }
        }
    } else {
        Err(format!("Not supported method {http_method}"))?
    }

    Ok(())
}

fn read_openapi(file: &Path) -> Result<OpenApi, Box<dyn Error>> {
    let mut content = String::new();
    File::open(file)?.read_to_string(&mut content)?;

    match file
        .extension()
        .ok_or("Not found extension")?
        .to_str()
        .unwrap()
    {
        "yml" => from_yml(&content),
        "yaml" => from_yml(&content),
        "json" => from_json(&content),
        _ => panic!("Not supported format."),
    }
}

fn from_json(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_json::from_str::<OpenApi>(content)?)
}

fn from_yml(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_yaml::from_str::<OpenApi>(content)?)
}

fn read_part_openapi(file: &Path) -> Result<PartOpenApi, Box<dyn Error>> {
    let mut content = String::new();
    File::open(file)?.read_to_string(&mut content)?;

    match file
        .extension()
        .ok_or("Not found extension")?
        .to_str()
        .unwrap()
    {
        "yml" => from_part_yml(&content),
        "yaml" => from_part_yml(&content),
        "json" => from_part_json(&content),
        _ => panic!("Not supported format."),
    }
}

fn from_part_json(content: &str) -> Result<PartOpenApi, Box<dyn Error>> {
    Ok(serde_json::from_str::<PartOpenApi>(content)?)
}

fn from_part_yml(content: &str) -> Result<PartOpenApi, Box<dyn Error>> {
    Ok(serde_yaml::from_str::<PartOpenApi>(content)?)
}

fn get_response_body(
    components: (&Path, &Option<Components>),
    r: &str,
) -> Result<(Components, Response), Box<dyn Error>> {
    if let Some(name) = r.strip_prefix("#/components/responses/") {
        let path = components.0;
        let body = components
            .1
            .as_ref()
            .ok_or(format!("Not found components {path:?}"))?
            .responses
            .as_ref()
            .ok_or(format!("Not found responses {path:?}"))?
            .get(name)
            .ok_or(format!("Not found responses  {path:?}:{name}"))?;
        match body {
            ReferenceOr::Value(v) => Ok((components.1.as_ref().unwrap().clone(), v.clone())),
            ReferenceOr::Ref(r) => get_response_body(components, &r.r#ref),
        }
    } else {
        get_remote_response_body(components.0, r)
    }
}

fn get_remote_response_body(
    path: &Path,
    r: &str,
) -> Result<(Components, Response), Box<dyn Error>> {
    if let Some((url, flags)) = r.rsplit_once('#') {
        if let Some((_, file)) = url.rsplit_once('/') {
            let mut path = PathBuf::from(path);
            path.pop();
            path.push(file);
            let model = read_part_openapi(&path)?;
            return get_response_body((&path, &model.components), &format!("#{flags}"));
        }
    }

    Err(format!("Not supported remote reference {r}"))?
}

fn get_schema(
    components: (&Path, &Option<Components>),
    r: &str,
) -> Result<(Components, Schema), Box<dyn Error>> {
    if let Some(name) = r.strip_prefix("#/components/schemas/") {
        let path = components.0;
        let schema = components
            .1
            .as_ref()
            .ok_or(format!("Not found components {path:?}"))?
            .schemas
            .as_ref()
            .ok_or(format!("Not found schemas {path:?}"))?
            .get(name)
            .ok_or(format!("Not found schemas  {path:?}:{name}"))?;
        Ok((components.1.as_ref().unwrap().clone(), schema.clone()))
    } else {
        get_remote_schema(components.0, r)
    }
}

fn get_remote_schema(path: &Path, r: &str) -> Result<(Components, Schema), Box<dyn Error>> {
    if let Some((url, flags)) = r.rsplit_once('#') {
        if let Some((_, file)) = url.rsplit_once('/') {
            let mut path = PathBuf::from(path);
            path.pop();
            path.push(file);
            let model = read_part_openapi(&path)?;
            return get_schema((&path, &model.components), &format!("#{flags}"));
        }
    }

    Err(format!("Not supported remote reference {r}"))?
}

fn dump_response_body(
    model: (&Path, &Option<Components>),
    body: &Response,
) -> Result<(), Box<dyn Error>> {
    if let Some(content) = &body.content {
        for (ty, media) in content {
            if ty.to_lowercase().contains("json") {
                dump_media(model, media)?;
            }
        }
    }

    Ok(())
}

fn dump_media(
    model: (&Path, &Option<Components>),
    media: &MediaType,
) -> Result<(), Box<dyn Error>> {
    if let Some(schema) = &media.schema {
        match &schema.r#ref {
            Some(r) => {
                let v = get_schema(model, r)?;
                dump_schema((model.0, &Some(v.0)), &v.1)?
            }
            _ => dump_schema(model, schema)?,
        }
    }

    Ok(())
}

fn dump_schema(model: (&Path, &Option<Components>), schema: &Schema) -> Result<(), Box<dyn Error>> {
    let model = get_schema_any(model, schema)?;
    let json = serde_json::to_string_pretty(&model)?;
    println!("{json}");

    Ok(())
}

fn get_schema_type(schema: &Schema) -> Result<&SchemaType, Box<dyn Error>> {
    match schema.r#type.as_ref() {
        Some(SchemaTypes::Unit(ty)) => Ok(ty),
        v => Err(format!("Not supported types {v:?}"))?,
    }
}

fn get_schema_any(
    model: (&Path, &Option<Components>),
    schema: &Schema,
) -> Result<Any, Box<dyn Error>> {
    if let Some(r) = schema.r#ref.as_ref() {
        let v = get_schema(model, r)?;
        return get_schema_any((model.0, &Some(v.0)), &v.1);
    }

    if let Some(any_of) = schema.any_of.as_ref() {
        if let Some(schema) = any_of.iter().next() {
            return get_schema_any(model, schema);
        }
    }

    let ty = get_schema_type(schema)?;
    let model = match ty {
        SchemaType::Null => Any::Null,
        SchemaType::Boolean => Any::String("Boolean".to_string()),
        SchemaType::Object => Any::Object(get_schema_hash(model, schema)?),
        SchemaType::Array => Any::Array(vec![get_schema_array(model, schema)?]),
        SchemaType::Number => Any::String("Number".to_string()),
        SchemaType::String => Any::String("String".to_string()),
        SchemaType::Integer => Any::String("Integer".to_string()),
    };
    Ok(model)
}

fn get_schema_array(
    model: (&Path, &Option<Components>),
    schema: &Schema,
) -> Result<Any, Box<dyn Error>> {
    let v = schema.items.as_ref().ok_or("Not found items")?;
    get_schema_any(model, v)
}

fn get_schema_hash(
    model: (&Path, &Option<Components>),
    schema: &Schema,
) -> Result<HashMap<String, Any>, Box<dyn Error>> {
    let mut obj = HashMap::new();

    if let Some(properties) = &schema.properties {
        for (name, property) in properties {
            obj.insert(name.to_string(), get_schema_any(model, property)?);
        }
    }

    if let Some(all_of) = &schema.all_of {
        for item in all_of {
            if let Any::Object(inner) = get_schema_any(model, item)? {
                for (k, v) in inner {
                    obj.insert(k, v);
                }
            } else {
                let ty = &item.r#type;
                println!("WARN: Not supported `allOf` type {ty:?}.");
            }
        }
    }

    Ok(obj)
}
