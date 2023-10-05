use openapi_spec_schema::model::Any;
use openapi_spec_schema::{
    MediaType, OpenApi, Reference, ReferenceOr, Response, Schema, SchemaType, SchemaTypes,
};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let path = PathBuf::from(args.nth(1).ok_or("Specify a file path.")?);
    let http_method = args.next().ok_or("Specify a HTTP method.")?;
    let http_path = args.next().ok_or("Specify a HTTP path.")?;
    let http_code = args
        .next()
        .map(|c| c.parse::<u16>().expect("Specify number"));

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
                    ReferenceOr::Value(v) => dump_response_body(&model, v)?,
                    ReferenceOr::Ref(r) => {
                        let v = get_response_body(&model, r)?;
                        dump_response_body(&model, v)?
                    }
                }
            } else {
                for code in res.statuses.values.keys() {
                    println!("{}", code);
                }
            }
        }
    } else {
        Err(format!("Not supported method {http_method}"))?
    }

    Ok(())
}

fn from_json(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_json::from_str::<OpenApi>(content)?)
}

fn from_yml(content: &str) -> Result<OpenApi, Box<dyn Error>> {
    Ok(serde_yaml::from_str::<OpenApi>(content)?)
}

fn get_response_body<'a>(
    model: &'a OpenApi,
    r: &'a Reference,
) -> Result<&'a Response, Box<dyn Error>> {
    if let Some(name) = r.r#ref.strip_prefix("#/components/responses/") {
        let body = model
            .components
            .as_ref()
            .ok_or("Not found components")?
            .responses
            .as_ref()
            .ok_or("Not found responses")?
            .get(name)
            .ok_or(format!("Not found responses {name}"))?;
        match body {
            ReferenceOr::Value(v) => Ok(v),
            ReferenceOr::Ref(r) => get_response_body(model, r),
        }
    } else {
        let name = r.r#ref.clone();
        Err(format!("Not supported remote reference {name}"))?
    }
}

fn get_schema<'a>(model: &'a OpenApi, r: &'a str) -> Result<&'a Schema, Box<dyn Error>> {
    if let Some(name) = r.strip_prefix("#/components/schemas/") {
        let schema = model
            .components
            .as_ref()
            .ok_or("Not found components")?
            .schemas
            .as_ref()
            .ok_or("Not found schemas")?
            .get(name)
            .ok_or(format!("Not found schemas {name}"))?;
        Ok(schema)
    } else {
        Err(format!("Not supported remote reference {r}"))?
    }
}

fn dump_response_body(model: &OpenApi, body: &Response) -> Result<(), Box<dyn Error>> {
    if let Some(content) = &body.content {
        for (ty, media) in content {
            if ty.to_lowercase().contains("json") {
                dump_media(model, media)?;
            }
        }
    }

    Ok(())
}

fn dump_media(model: &OpenApi, media: &MediaType) -> Result<(), Box<dyn Error>> {
    if let Some(schema) = &media.schema {
        match &schema.r#ref {
            Some(r) => {
                let v = get_schema(model, r)?;
                dump_schema(model, v)?
            }
            _ => dump_schema(model, schema)?,
        }
    }

    Ok(())
}

fn dump_schema(model: &OpenApi, schema: &Schema) -> Result<(), Box<dyn Error>> {
    let model = get_schema_any(model, schema)?;
    let json = serde_json::to_string_pretty(&model)?;
    println!("{}", json);

    Ok(())
}

fn get_schema_type(schema: &Schema) -> Result<&SchemaType, Box<dyn Error>> {
    match schema.r#type.as_ref() {
        Some(SchemaTypes::Unit(ty)) => Ok(ty),
        v => Err(format!("Not supported types {v:?}"))?,
    }
}

fn get_schema_any(model: &OpenApi, schema: &Schema) -> Result<Any, Box<dyn Error>> {
    if let Some(r) = schema.r#ref.as_ref() {
        let v = get_schema(model, r)?;
        return get_schema_any(model, v);
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

fn get_schema_array(model: &OpenApi, schema: &Schema) -> Result<Any, Box<dyn Error>> {
    let v = schema.items.as_ref().ok_or("Not found items")?;
    get_schema_any(model, v)
}

fn get_schema_hash(
    model: &OpenApi,
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
