use super::error::Error;
use super::SchemaItem;
use openapi_spec_schema::{Schema, SchemaType, SchemaTypes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::str::FromStr;

pub fn gen(schemas: &[SchemaItem]) -> Result<String, Error> {
    let config = Config { schemas };

    let mut token_structs = vec![];
    for item in config.schemas {
        if item.schema.all_of.is_some() {
            panic!("Not supported `allOf`.");
        }

        if item.schema.one_of.is_some() {
            panic!("Not supported `oneOf`.");
        }

        let struct_name = upper_camel_case(&item.schema_name);
        let struct_ident = format_ident!("{struct_name}");

        if let Some(any_of) = item.schema.any_of.as_ref() {
            let mut variants = vec![];
            for variant_schema in any_of {
                let r = variant_schema.r#ref.as_deref().unwrap();
                let variant_schema_def = config.get_def_by_url(&item.file_name, r)?;
                variants.push(variant_schema_def.schema_name.clone());
            }

            variants.sort();

            let variant_idents = variants.iter().map(|v| {
                let variant_ident = format_ident!("{}", upper_camel_case(v));
                quote! {
                    #variant_ident(#variant_ident)
                }
            });

            token_structs.push(quote! {
                #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
                #[serde(untagged)]
                pub enum #struct_ident {
                    #(#variant_idents),*
                }
            });
        } else if let Some(enum_values) = item.schema.r#enum.as_ref() {
            let mut variants = enum_values.to_vec();

            variants.sort();

            let variant_idents = variants.iter().map(|v| {
                let variant_ident = format_ident!("{}", upper_camel_case(v));
                quote! {
                    #[serde(rename = #v)]
                    #variant_ident
                }
            });

            token_structs.push(quote! {
                #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
                pub enum #struct_ident {
                    #[default]
                    #(#variant_idents),*
                }
            });
        } else if !primitive(&item.schema) {
            gen_struct(&config, item, &mut token_structs)?;
        }
    }

    let model = quote! {
        use serde::{Deserialize, Serialize};

        #(#token_structs)*
    };

    Ok(model.to_string())
}

fn gen_struct(
    config: &Config,
    item: &SchemaItem,
    token_structs: &mut Vec<TokenStream>,
) -> Result<(), Error> {
    let struct_name = upper_camel_case(&item.schema_name);
    let struct_ident = format_ident!("{struct_name}");

    let mut property_info = vec![];
    if let Some(properties) = item.schema.properties.as_ref() {
        for (prop_name, prop_schema) in properties {
            let mut derive = quote! { #[serde(rename = #prop_name)] };

            let field_name = snake_case(prop_name);
            let field_ident = if rust_reserved(&field_name) {
                format_ident!("r#{}", snake_case(prop_name))
            } else {
                format_ident!("{}", snake_case(prop_name))
            };

            let mut field_ty = if anonymous_ty(prop_schema) {
                let ty_name = upper_camel_case(&format!("{struct_name}_{prop_name}"));
                let ty_name_ident = format_ident!("{ty_name}");

                let anonymous = SchemaItem {
                    file_name: item.file_name.clone(),
                    schema: prop_schema.clone(),
                    schema_name: ty_name,
                };

                gen_struct(config, &anonymous, token_structs)?;

                quote! { #ty_name_ident }
            } else {
                schema_ty(config, &item.file_name, prop_schema)?
            };

            if !required(&item.schema, prop_name) {
                derive = quote! { #[serde(skip_serializing_if = "Option::is_none", rename = #prop_name)] };
                field_ty = quote! { Option<#field_ty> };
            } else if optional(prop_schema) {
                field_ty = quote! { Option<#field_ty> };
            }

            property_info.push(PropertyInfo {
                name: field_name,
                token: quote! {
                    #derive
                    pub #field_ident: #field_ty
                },
            });
        }
    }

    property_info.sort_unstable_by_key(|p| p.name.clone());
    let token_properties = property_info.iter().map(|p| &p.token);

    token_structs.push(quote! {
        #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
        pub struct #struct_ident {
            #(#token_properties),*
        }
    });

    Ok(())
}

fn schema_ty(config: &Config, file_name: &str, schema: &Schema) -> Result<TokenStream, Error> {
    match schema.r#type.as_ref() {
        Some(SchemaTypes::Unit(ty)) => match ty {
            SchemaType::Null => panic!("Not supported types."),
            SchemaType::Boolean => Ok(quote! { bool }),
            SchemaType::Object => schema_object_ty(config, file_name, schema),
            SchemaType::Array => schema_array_ty(config, file_name, schema),
            SchemaType::Number => Ok(quote! { f64 }),
            SchemaType::String => Ok(quote! { String }),
            SchemaType::Integer => Ok(quote! { i64 }),
        },
        None if schema.r#ref.is_some() => schema_object_ty(config, file_name, schema),
        _ => panic!("Not supported types."),
    }
}

fn schema_array_ty(
    config: &Config,
    file_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    let elem_schema = schema.items.as_ref().unwrap();
    let elem_ty = schema_ty(config, file_name, elem_schema)?;
    Ok(quote! { Vec<#elem_ty> })
}

fn schema_object_ty(
    config: &Config,
    file_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    let schema_ref = schema.r#ref.as_deref().unwrap();
    let schema_def = config.get_def_by_url(file_name, schema_ref)?;
    if primitive(&schema_def.schema) {
        return schema_ty(config, &schema_def.file_name, &schema_def.schema);
    }

    let ref_name = upper_camel_case(&schema_def.schema_name);
    let ref_ident = format_ident!("{ref_name}");
    Ok(quote! { #ref_ident })
}

fn anonymous_ty(schema: &Schema) -> bool {
    schema.r#type == Some(SchemaTypes::Unit(SchemaType::Object)) && schema.r#ref.is_none()
}

fn primitive(schema: &Schema) -> bool {
    match schema.r#type.as_ref() {
        Some(SchemaTypes::Unit(ty)) => match ty {
            SchemaType::Null => true,
            SchemaType::Boolean => true,
            SchemaType::Object => false,
            SchemaType::Array => false,
            SchemaType::Number => true,
            SchemaType::String => true,
            SchemaType::Integer => true,
        },
        None => false,
        _ => panic!("Not supported types."),
    }
}

fn optional(prop_schema: &Schema) -> bool {
    prop_schema.nullable.unwrap_or(false)
}

fn required(ty: &Schema, prop_name: &str) -> bool {
    if let Some(required) = ty.required.as_deref() {
        if required.iter().any(|p| p == prop_name) {
            return true;
        }
    }

    false
}

fn upper_camel_case(value: &str) -> String {
    value
        .split(&['_', '+', '-', '.', '@', '#', ' ', '/', ':'][..])
        .map(|v| {
            v.chars()
                .enumerate()
                .map(|(index, c)| match index {
                    0 => c.to_ascii_uppercase(),
                    _ => c,
                })
                .collect::<String>()
        })
        .fold("".to_string(), |mut acc, x| {
            match u64::from_str(&x) {
                Ok(n) => acc.push_str(&format!("N{n}")),
                _ => acc.push_str(&x),
            };

            acc
        })
}

fn snake_case(value: &str) -> String {
    let v = value
        .split(&['_', '+', '-', '.', '@', '#', ' ', '/', ':'][..])
        .flat_map(|v| {
            let mut words = vec![];
            let mut index = 0;
            let mut upper_char = false;

            for (next, _) in v.match_indices(char::is_uppercase) {
                if !upper_char || (next - index) > 1 {
                    words.push(v[index..next].to_ascii_lowercase());
                } else {
                    // combine upper-case char sequence.
                    words
                        .last_mut()
                        .unwrap()
                        .push_str(&v[index..next].to_ascii_lowercase());
                }

                upper_char = (next - index) == 1;
                index = next;
            }

            match index {
                0 => words.push(v.to_ascii_lowercase()),
                _ => {
                    if !upper_char || (v.len() - index) > 1 {
                        words.push(v[index..].to_ascii_lowercase());
                    } else {
                        // combine upper-case char sequence.
                        words
                            .last_mut()
                            .unwrap()
                            .push_str(&v[index..].to_ascii_lowercase());
                    }
                }
            }

            words
        })
        .filter(|v| !v.is_empty())
        .collect::<Vec<String>>()
        .join("_")
        .trim_matches('_')
        .to_string();

    match u64::from_str(&v) {
        Ok(n) => format!("N{n}"),
        _ => v,
    }
}

// ---------------------------------------------------------------------------

// https://doc.rust-lang.org/book/appendix-01-keywords.html
const KEYWORDS_RUST: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while",
];

pub fn rust_reserved(keyword: &str) -> bool {
    KEYWORDS_RUST.iter().any(|&k| k == keyword)
}

// ---------------------------------------------------------------------------

struct Config<'a> {
    schemas: &'a [SchemaItem],
}

impl<'a> Config<'a> {
    fn get_def_by_ref(&self, r: &str) -> Result<&'a SchemaItem, Error> {
        self.schemas
            .iter()
            .find(|&s| s.r#ref() == r)
            .ok_or(Error::not_found_schema(r))
    }

    fn get_def_by_url(&self, file_name: &str, uri: &str) -> Result<&'a SchemaItem, Error> {
        let (url, component) = uri.rsplit_once('#').ok_or(Error::invalid_uri(uri))?;
        let (_, file_name) = url.rsplit_once('/').unwrap_or(("", file_name));
        let (_, schema_name) = component
            .rsplit_once('/')
            .ok_or(Error::invalid_component_path(uri))?;
        self.get_def_by_ref(&format!("{file_name}#{schema_name}"))
    }
}

// ---------------------------------------------------------------------------

struct PropertyInfo {
    name: String,
    token: TokenStream,
}
