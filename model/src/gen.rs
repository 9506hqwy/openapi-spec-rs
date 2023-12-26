use super::error::Error;
use super::{anonymous_ty, SchemaItem};
use openapi_spec_schema::{Schema, SchemaType, SchemaTypes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

pub fn gen(output: &Path, schemas: &[SchemaItem]) -> Result<(), Error> {
    let config = Config { schemas };

    let mut structs = vec![];
    let mut progress = 0usize;
    for item in config.schemas {
        if item.schema.all_of.is_some() {
            panic!("Not supported `allOf`.");
        }

        if item.schema.one_of.is_some() {
            panic!("Not supported `oneOf`.");
        }

        if item.schema.any_of.is_some() {
            structs.push(gen_newtype_variant(&config, item)?);
        } else if item.schema.r#enum.is_some() {
            structs.push(gen_unit_variant(&config, item)?);
        } else if !primitive(&item.schema) {
            structs.append(&mut gen_struct(&config, item)?);
        }

        let percentage = structs.len() * 100 / config.schemas.len();
        if progress != percentage {
            progress = percentage;
            println!("Generated {}%", progress);
        }
    }

    let mut use_modules = vec![];
    for domain_name in domain_names(&structs) {
        let domain_types = structs
            .iter()
            .filter(|s| s.domain == domain_name)
            .collect::<Vec<&StructInfo>>();

        let output_domain = output.join(&domain_name);
        if !domain_name.is_empty() {
            fs::create_dir(&output_domain)?;
        }

        let mut domain_modules = vec![];
        for module_name in module_names(&structs) {
            if module_name.is_empty() {
                continue;
            }

            let module_types = domain_types
                .iter()
                .filter(|s| s.module == module_name)
                .copied()
                .collect::<Vec<&StructInfo>>();
            if module_types.is_empty() {
                continue;
            }

            let mut token_versions = vec![];
            for version in versions(&module_types) {
                if version.is_empty() {
                    continue;
                }

                let mut version_types = module_types
                    .iter()
                    .filter(|s| s.version_str() == version)
                    .copied()
                    .collect::<Vec<&StructInfo>>();
                if version_types.is_empty() {
                    continue;
                }

                version_types.sort_unstable_by_key(|s| &s.name);
                let tokens = version_types.iter().map(|s| &s.token);

                let version_ident = format_ident!("{version}");
                token_versions.push(quote! {
                    pub mod #version_ident {
                        use serde::{Deserialize, Serialize};

                        #(#tokens)*
                    }
                });
            }

            let mut m_types = module_types
                .iter()
                .filter(|s| s.version_str().is_empty())
                .copied()
                .collect::<Vec<&StructInfo>>();

            let use_stmt = if m_types.is_empty() {
                quote! {}
            } else {
                quote! {
                    use serde::{Deserialize, Serialize};
                }
            };

            let token = if m_types.is_empty() {
                quote! {}
            } else {
                m_types.sort_unstable_by_key(|s| &s.name);
                let tokens = m_types.iter().map(|s| &s.token);

                quote! {
                    #(#tokens)*
                }
            };

            let module_ident = format_ident!("{module_name}");
            let model = quote! {
                #use_stmt

                #token

                #(#token_versions)*
            };

            if domain_name.is_empty() {
                use_modules.push(quote! {
                    pub mod #module_ident;
                });
            } else {
                domain_modules.push(quote! {
                    pub mod #module_ident;
                });
            }

            let file_path = output_domain.join(format!("{module_name}.rs"));
            write_code(&file_path, &model)?;
        }

        // create `mod.rs`

        if !domain_name.is_empty() {
            let mut m_types = domain_types
                .iter()
                .filter(|s| s.module.is_empty())
                .copied()
                .collect::<Vec<&StructInfo>>();

            let use_stmt = if m_types.is_empty() {
                quote! {}
            } else {
                quote! {
                    use serde::{Deserialize, Serialize};
                }
            };

            let token = if m_types.is_empty() {
                quote! {}
            } else {
                m_types.sort_unstable_by_key(|s| &s.name);
                let tokens = m_types.iter().map(|s| &s.token);

                quote! {
                    #(#tokens)*
                }
            };

            let file_path = output_domain.join("mod.rs");
            let model = quote! {
                #(#domain_modules)*

                #use_stmt

                #token
            };
            write_code(&file_path, &model)?;

            let domain_ident = format_ident!("{domain_name}");
            use_modules.push(quote! {
                pub mod #domain_ident;
            });
        }
    }

    // create `lib.rs`

    let mut s_types = structs
        .iter()
        .filter(|s| s.domain.is_empty())
        .filter(|s| s.module.is_empty())
        .collect::<Vec<&StructInfo>>();

    let use_stmt = if s_types.is_empty() {
        quote! {}
    } else {
        quote! {
            use serde::{Deserialize, Serialize};
        }
    };

    let token = if s_types.is_empty() {
        quote! {}
    } else {
        s_types.sort_unstable_by_key(|s| &s.name);
        let tokens = s_types.iter().map(|s| &s.token);

        quote! {
            #(#tokens)*
        }
    };

    let model = quote! {
        #(#use_modules)*

        #use_stmt

        #token
    };

    let file_path = output.join("lib.rs");
    write_code(&file_path, &model)?;

    Ok(())
}

fn write_code(file_path: &Path, token: &TokenStream) -> Result<(), Error> {
    let mut file = File::create(file_path)?;
    let code = token.to_string();

    file.write_all(code.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;

    Ok(())
}

fn gen_unit_variant(config: &Config, item: &SchemaItem) -> Result<StructInfo, Error> {
    let struct_name = config.types_name(&item.schema_name);
    let struct_ident = format_ident!("{struct_name}");

    let mut variants = item.schema.r#enum.as_ref().unwrap().to_vec();

    variants.sort();

    let variant_idents = variants.iter().map(|v| {
        let variant_ident = format_ident!("{}", config.enum_variants_name(v));
        quote! {
            #[serde(rename = #v)]
            #variant_ident
        }
    });

    let (module_name, version) = config.modules_name(&item.schema_name);

    Ok(StructInfo {
        domain: item.domain_name.clone(),
        module: module_name,
        version,
        name: struct_name,
        token: quote! {
            #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
            pub enum #struct_ident {
                #[default]
                #(#variant_idents),*
            }
        },
    })
}

fn gen_newtype_variant(config: &Config, item: &SchemaItem) -> Result<StructInfo, Error> {
    let struct_name = config.types_name(&item.schema_name);
    let struct_ident = format_ident!("{struct_name}");

    let mut variants = vec![];
    for variant_schema in item.schema.any_of.as_ref().unwrap() {
        let r = variant_schema.r#ref.as_deref().unwrap();
        let variant_schema_def =
            config.get_def_by_url(&item.domain_name, &item.schema_file_name, r)?;
        let (_, version) = config.modules_name(&variant_schema_def.schema_name);
        variants.push((
            version_num(version),
            variant_schema_def.domain_name.clone(),
            variant_schema_def.schema_name.clone(),
        ));
    }

    variants.sort_unstable_by_key(|v| v.0);
    variants.reverse();

    let variant_idents = variants.iter().map(|v| {
        let variant_name = if v.0 == 0 {
            config.enum_variants_name(&v.2)
        } else {
            config.enum_variants_name(&format!("v{:06}", v.0))
        };
        let variant_ident = format_ident!("{}", variant_name);
        let ref_ty_ident = config.ref_types_name(&v.1, &v.2);
        quote! {
            #variant_ident(#ref_ty_ident)
        }
    });

    let (module_name, version) = config.modules_name(&item.schema_name);

    Ok(StructInfo {
        domain: item.domain_name.clone(),
        module: module_name,
        version,
        name: struct_name,
        token: quote! {
            #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
            #[serde(untagged)]
            pub enum #struct_ident {
                #(#variant_idents),*
            }
        },
    })
}

fn gen_struct(config: &Config, item: &SchemaItem) -> Result<Vec<StructInfo>, Error> {
    let struct_name = config.types_name(&item.schema_name);
    let struct_ident = format_ident!("{struct_name}");

    let mut structs = vec![];
    let mut property_info = vec![];
    if let Some(properties) = item.schema.properties.as_ref() {
        for (prop_name, prop_schema) in properties {
            let mut derive = quote! { #[serde(rename = #prop_name)] };

            let field_name = config.fields_name(prop_name);
            let field_ident = if rust_reserved(&field_name) {
                format_ident!("r#{}", field_name)
            } else {
                format_ident!("{}", field_name)
            };

            let mut field_ty = if anonymous_ty(prop_schema) {
                let schema_name = &item.schema_name;

                let anonymous = SchemaItem {
                    domain_name: item.domain_name.clone(),
                    schema_file_name: item.schema_file_name.clone(),
                    schema: prop_schema.clone(),
                    // join '-' because of splitting '_' for name conversion using `Config` struct.
                    schema_name: format!("{schema_name}-{prop_name}"),
                };

                structs.append(&mut gen_struct(config, &anonymous)?);

                config.ref_types_name(&anonymous.domain_name, &anonymous.schema_name)
            } else {
                schema_ty(
                    config,
                    &item.domain_name,
                    &item.schema_file_name,
                    prop_schema,
                )?
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

    let (module_name, version) = config.modules_name(&item.schema_name);

    structs.push(StructInfo {
        domain: item.domain_name.clone(),
        module: module_name,
        version,
        name: struct_name,
        token: quote! {
            #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
            pub struct #struct_ident {
                #(#token_properties),*
            }
        },
    });

    Ok(structs)
}

fn schema_ty(
    config: &Config,
    domain: &str,
    file_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    match schema.r#type.as_ref() {
        Some(SchemaTypes::Unit(ty)) => match ty {
            SchemaType::Null => panic!("Not supported types."),
            SchemaType::Boolean => Ok(quote! { bool }),
            SchemaType::Object => schema_object_ty(config, domain, file_name, schema),
            SchemaType::Array => schema_array_ty(config, domain, file_name, schema),
            SchemaType::Number => Ok(quote! { f64 }),
            SchemaType::String => Ok(quote! { String }),
            SchemaType::Integer => Ok(quote! { i64 }),
        },
        None if schema.r#ref.is_some() => schema_object_ty(config, domain, file_name, schema),
        _ => panic!("Not supported types."),
    }
}

fn schema_array_ty(
    config: &Config,
    domain: &str,
    file_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    let elem_schema = schema.items.as_ref().unwrap();
    let elem_ty = schema_ty(config, domain, file_name, elem_schema)?;
    Ok(quote! { Vec<#elem_ty> })
}

fn schema_object_ty(
    config: &Config,
    domain: &str,
    file_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    let schema_ref = schema.r#ref.as_deref().unwrap();
    let schema_def = config.get_def_by_url(domain, file_name, schema_ref)?;
    if primitive(&schema_def.schema) {
        return schema_ty(
            config,
            &schema_def.domain_name,
            &schema_def.schema_file_name,
            &schema_def.schema,
        );
    }

    Ok(config.ref_types_name(&schema_def.domain_name, &schema_def.schema_name))
}

fn primitive(schema: &Schema) -> bool {
    if schema.r#enum.is_some() {
        return false;
    }

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
        // Change word because snake-casing not work well.
        .replace("ETag", "Etag")
        .replace("FCoE", "FcOe")
        .replace("GiB", "Gib")
        .replace("I2C", "I2c")
        .replace("IDs", "Ids")
        .replace("IPv4", "Ipv4")
        .replace("IPv6", "Ipv6")
        .replace("KiB", "Kib")
        .replace("kVAh", "Kvah")
        .replace("kVARh", "Kvarh")
        .replace("LoS", "Los")
        .replace("MiB", "Mib")
        .replace("MHz", "Mhz")
        .replace("NVMe", "Nvme")
        .replace("NvmeoF", "Nvme_Of")
        .replace("OAuth", "Oauth")
        .replace("PCIe", "Pcie")
        .replace("QoS", "Qos")
        .replace("VLAN", "Vlan")
        .replace("VLan", "Vlan")
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

    fn get_def_by_url(
        &self,
        domain_name: &str,
        file_name: &str,
        uri: &str,
    ) -> Result<&'a SchemaItem, Error> {
        let (url, component) = uri.rsplit_once('#').ok_or(Error::invalid_uri(uri))?;
        let domain_name = self.domain(url, domain_name);
        let (_, file_name) = url.rsplit_once('/').unwrap_or(("", file_name));
        let (_, schema_name) = component
            .rsplit_once('/')
            .ok_or(Error::invalid_component_path(uri))?;
        self.get_def_by_ref(&format!("{domain_name}/{file_name}#{schema_name}"))
    }

    // NOTICE: consider name conversion for redfish schema only.

    fn domain(&self, url: &str, domain_name: &str) -> String {
        let domain_name = url.split('/').nth(4).unwrap_or(domain_name);
        if domain_name == "v1" {
            "".to_string()
        } else {
            domain_name.to_string()
        }
    }

    fn modules_name(&self, source: &str) -> (String, Option<(u8, u8, u8)>) {
        let (value, _) = source.rsplit_once('_').unwrap_or_default();
        let (module, version_str) = value.split_once('_').unwrap_or((value, ""));

        let mut version = None;
        if !version_str.is_empty() {
            let mut v = version_str
                .strip_prefix('v')
                .unwrap_or(version_str)
                .splitn(3, '_')
                .map(u8::from_str)
                .map(|v| v.unwrap());
            version = Some((v.next().unwrap(), v.next().unwrap(), v.next().unwrap()));
        }

        (snake_case(module), version)
    }

    fn types_name(&self, source: &str) -> String {
        let (_, value) = source.rsplit_once('_').unwrap_or(("", source));
        upper_camel_case(value)
    }

    fn enum_variants_name(&self, source: &str) -> String {
        upper_camel_case(source)
    }

    fn fields_name(&self, source: &str) -> String {
        snake_case(source)
    }

    fn ref_types_name(&self, domain: &str, source: &str) -> TokenStream {
        let (module, version) = self.modules_name(source);
        let ty_name = self.types_name(source);

        let mut type_names = vec!["crate".to_string()];

        if !domain.is_empty() {
            type_names.push(domain.to_string());
        }

        if !module.is_empty() {
            type_names.push(module);
        }

        if let Some(v) = version {
            type_names.push(format!("v{}_{}_{}", v.0, v.1, v.2));
        }

        if !ty_name.is_empty() {
            type_names.push(ty_name);
        }

        let type_idents = type_names.iter().map(|n| format_ident!("{n}"));

        quote! {
            #(#type_idents)::*
        }
    }
}

// ---------------------------------------------------------------------------

struct StructInfo {
    domain: String,
    module: String,
    version: Option<(u8, u8, u8)>,
    name: String,
    token: TokenStream,
}

impl StructInfo {
    fn version_num(&self) -> u32 {
        version_num(self.version)
    }

    fn version_str(&self) -> String {
        match self.version {
            Some(v) => format!("v{}_{}_{}", v.0, v.1, v.2),
            _ => "".to_string(),
        }
    }
}

struct PropertyInfo {
    name: String,
    token: TokenStream,
}

fn version_num(version: Option<(u8, u8, u8)>) -> u32 {
    match version {
        Some(v) => (v.0 as u32) * 10000 + (v.1 as u32) * 100 + (v.2 as u32),
        _ => 0,
    }
}

fn domain_names(items: &[StructInfo]) -> Vec<String> {
    let domain_names = items
        .iter()
        .map(|s| &s.domain)
        .collect::<HashSet<&String>>();

    let mut domain_names = domain_names.into_iter().cloned().collect::<Vec<String>>();
    domain_names.sort();
    domain_names
}

fn module_names(items: &[StructInfo]) -> Vec<String> {
    let module_names = items
        .iter()
        .map(|s| &s.module)
        .collect::<HashSet<&String>>();

    let mut module_names = module_names.into_iter().cloned().collect::<Vec<String>>();
    module_names.sort();
    module_names
}

fn versions(items: &[&StructInfo]) -> Vec<String> {
    let versions = items
        .iter()
        .map(|&s| (s.version_num(), s.version_str()))
        .collect::<Vec<(u32, String)>>();

    let keys = versions.iter().map(|v| v.0).collect::<HashSet<u32>>();

    let mut keys = keys.into_iter().collect::<Vec<u32>>();
    keys.sort();

    keys.iter()
        .map(|&k| versions.iter().find(|v| v.0 == k).unwrap().1.clone())
        .collect::<Vec<String>>()
}
