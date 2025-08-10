use super::error::Error;
use super::{Config, SchemaItem, anonymous_ty};
use openapi_spec_schema::{Schema, SchemaType, SchemaTypes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

pub fn gen_code(output: &Path, schemas: &[SchemaItem]) -> Result<(), Error> {
    let config = Config { schemas };

    let mut structs = vec![];
    let mut progress = 0usize;
    for item in config.schemas {
        if item.schema.all_of.is_some() {
            panic!("Not supported `allOf`.");
        }

        if item.schema.r#ref.is_some() {
            structs.push(get_type_alias(&config, item)?);
        } else if item.schema.any_of.is_some() {
            structs.push(gen_newtype_variant(&config, item)?);
        } else if item.schema.r#enum.is_some() {
            structs.push(gen_unit_variant(&config, item)?);
        } else if item.schema.one_of.is_some() {
            structs.push(gen_newtype_variant(&config, item)?);
        } else if !primitive(&item.schema) {
            structs.append(&mut gen_struct(&config, item)?);
        }

        let percentage = structs.len() * 100 / config.schemas.len();
        if progress != percentage {
            progress = percentage;
            println!("Generated {progress}%");
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

            let use_stmt = if m_types.iter().any(|t| t.use_serde) {
                quote! {
                    use serde::{Deserialize, Serialize};
                }
            } else {
                quote! {}
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

            let use_stmt = if m_types.iter().any(|t| t.use_serde) {
                quote! {
                    use serde::{Deserialize, Serialize};
                }
            } else {
                quote! {}
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

    let use_stmt = if s_types.iter().any(|t| t.use_serde) {
        quote! {
            use serde::{Deserialize, Serialize};
        }
    } else {
        quote! {}
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

/// ファイルに書き出す。
fn write_code(file_path: &Path, token: &TokenStream) -> Result<(), Error> {
    let mut file = File::create(file_path)?;
    let code = token.to_string();

    file.write_all(code.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;

    Ok(())
}

/// 型エイリアスのコードを生成する。
fn get_type_alias(config: &Config, item: &SchemaItem) -> Result<StructInfo, Error> {
    let struct_name = config.types_name(&item.schema_name, item.duplicated);
    let struct_ident = format_ident!("{struct_name}");

    let r = item.schema.r#ref.as_deref().unwrap();
    let ty_item = config.get_def_by_url(&item.domain_name, &item.schema_file_name, r)?;

    let ty_token = schema_ref_type_name(
        config,
        &ty_item.domain_name,
        &ty_item.schema_name,
        ty_item.duplicated,
    );

    let (module_name, version) = config.modules_name(&item.schema_name);

    Ok(StructInfo {
        domain: item.domain_name.clone(),
        module: module_name,
        version,
        name: struct_name,
        token: quote! {
            pub type #struct_ident = #ty_token;
        },
        use_serde: false,
    })
}

/// unit variant のコードを生成する。
fn gen_unit_variant(config: &Config, item: &SchemaItem) -> Result<StructInfo, Error> {
    let struct_name = config.types_name(&item.schema_name, item.duplicated);
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
        use_serde: true,
    })
}

/// newtype variant のコードを生成する。
fn gen_newtype_variant(config: &Config, item: &SchemaItem) -> Result<StructInfo, Error> {
    let struct_name = config.types_name(&item.schema_name, item.duplicated);
    let struct_ident = format_ident!("{struct_name}");

    let mut variants = vec![];

    if let Some(any_of) = item.schema.any_of.as_ref() {
        for variant_schema in any_of {
            // TODO: anonymous
            let r = variant_schema.r#ref.as_deref().unwrap();
            let variant_schema_def =
                config.get_def_by_url(&item.domain_name, &item.schema_file_name, r)?;
            let (_, version) = config.modules_name(&variant_schema_def.schema_name);
            variants.push((version_num(version), variant_schema_def));
        }
    }

    if let Some(one_of) = item.schema.one_of.as_ref() {
        for (i, variant_schema) in one_of.iter().enumerate() {
            if let Some(r) = variant_schema.r#ref.as_deref() {
                let variant_schema_def =
                    config.get_def_by_url(&item.domain_name, &item.schema_file_name, r)?;
                let (_, version) = config.modules_name(&variant_schema_def.schema_name);
                variants.push((version_num(version), variant_schema_def));
            } else if variant_schema.r#enum.is_some() {
                let schema_name = format!("{}-{}", item.schema_name, i);
                let variant_schema_def = config.get_def_by_name(&item.domain_name, &schema_name)?;
                variants.push((i as u32, variant_schema_def));
            } else {
                dbg!(&item.schema);
                panic!(
                    "Not supported item of `oneOf` ({}/{}).",
                    item.schema_file_name, item.schema_name
                );
            }
        }
    }

    variants.sort_unstable_by_key(|v| v.0);
    variants.reverse();

    let variant_idents = variants.iter().map(|v| {
        let variant_name = if v.0 == 0 {
            // バージョンがない場合はフラグメント
            config.enum_variants_name(&v.1.schema_name)
        } else {
            // バージョンがある場合はバージョン
            config.enum_variants_name(&format!("v{:06}", v.0))
        };
        let variant_ident = format_ident!("{}", variant_name);

        let ref_ty_ident = schema_ty(
            config,
            &v.1.domain_name,
            &v.1.schema_file_name,
            &v.1.schema_name,
            "",
            &v.1.schema,
        )
        .unwrap();

        quote! {
            #variant_ident(#ref_ty_ident)
        }
    });

    let default_impl = variants.first().map(|v| {
        let variant_name = if v.0 == 0 {
            // バージョンがない場合はフラグメント
            config.enum_variants_name(&v.1.schema_name)
        } else {
            // バージョンがある場合はバージョン
            config.enum_variants_name(&format!("v{:06}", v.0))
        };
        let variant_ident = format_ident!("{}", variant_name);

        quote! {
            impl Default for #struct_ident {
                fn default() -> Self {
                    Self::#variant_ident(Default::default())
                }
            }
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

            #default_impl
        },
        use_serde: true,
    })
}

/// struct のコードを生成する。
fn gen_struct(config: &Config, item: &SchemaItem) -> Result<Vec<StructInfo>, Error> {
    let struct_name = config.types_name(&item.schema_name, item.duplicated);
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
                let property_name = format!("{schema_name}-{prop_name}");
                let property_def = config.get_def_by_name(&item.domain_name, &property_name)?;

                schema_ref_type_name(
                    config,
                    &property_def.domain_name,
                    &property_def.schema_name,
                    property_def.duplicated,
                )
            } else {
                schema_ty(
                    config,
                    &item.domain_name,
                    &item.schema_file_name,
                    &item.schema_name,
                    prop_name,
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
        use_serde: true,
    });

    Ok(structs)
}

/// スキーマの Rust 型を取得する。
fn schema_ty(
    config: &Config,
    domain: &str,
    file_name: &str,
    struct_name: &str,
    prop_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    match schema.r#type.as_ref() {
        Some(SchemaTypes::Unit(ty)) => match ty {
            SchemaType::Null => panic!("Not supported types."),
            SchemaType::Boolean => Ok(quote! { bool }),
            SchemaType::Object => {
                schema_object_ty(config, domain, file_name, struct_name, prop_name, schema)
            }
            SchemaType::Array => {
                schema_array_ty(config, domain, file_name, struct_name, prop_name, schema)
            }
            SchemaType::Number => Ok(quote! { f64 }),
            SchemaType::String => {
                if schema.r#enum.is_some() {
                    let schema_def = config.get_def_by_name(domain, struct_name)?;
                    Ok(schema_ref_type_name(
                        config,
                        &schema_def.domain_name,
                        &schema_def.schema_name,
                        schema_def.duplicated,
                    ))
                } else {
                    Ok(quote! { String })
                }
            }
            SchemaType::Integer => Ok(quote! { i64 }),
        },
        None => schema_object_ty(config, domain, file_name, struct_name, prop_name, schema),
        _ => panic!("Not supported types."),
    }
}

/// 配列型のスキーマの Rust 型を取得する。
fn schema_array_ty(
    config: &Config,
    domain: &str,
    file_name: &str,
    struct_name: &str,
    prop_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    let elem_schema = schema.items.as_ref().unwrap();
    let elem_ty = schema_ty(
        config,
        domain,
        file_name,
        struct_name,
        prop_name,
        elem_schema,
    )?;
    Ok(quote! { Vec<#elem_ty> })
}

/// オブジェクト型のスキーマの Rust 型を取得する。
fn schema_object_ty(
    config: &Config,
    domain: &str,
    file_name: &str,
    struct_name: &str,
    prop_name: &str,
    schema: &Schema,
) -> Result<TokenStream, Error> {
    match schema.r#ref.as_deref() {
        Some(schema_ref) => {
            let schema_def = config.get_def_by_url(domain, file_name, schema_ref)?;
            if primitive(&schema_def.schema) {
                return schema_ty(
                    config,
                    &schema_def.domain_name,
                    &schema_def.schema_file_name,
                    struct_name,
                    prop_name,
                    &schema_def.schema,
                );
            }

            Ok(schema_ref_type_name(
                config,
                &schema_def.domain_name,
                &schema_def.schema_name,
                schema_def.duplicated,
            ))
        }
        _ => {
            let ty_name = format!("{struct_name}-{prop_name}");
            Ok(schema_ref_type_name(config, domain, &ty_name, false))
        }
    }
}

/// 指定したスキーマの名前から Rust 型を取得する。
fn schema_ref_type_name(
    config: &Config,
    domain: &str,
    source: &str,
    duplicated: bool,
) -> TokenStream {
    let type_names = config.ref_types_name(domain, source, duplicated);
    let type_idents = type_names.iter().map(|n| format_ident!("{n}"));

    quote! {
        #(#type_idents)::*
    }
}

/// スキーマが Rust プリミティブ型かどうか判定する。
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

/// スキーマが null 許容かどうか判定する。
fn optional(prop_schema: &Schema) -> bool {
    prop_schema.nullable.unwrap_or(false)
}

/// プロパティが必須かどうか判定する。
fn required(ty: &Schema, prop_name: &str) -> bool {
    if let Some(required) = ty.required.as_deref() {
        if required.iter().any(|p| p == prop_name) {
            return true;
        }
    }

    false
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
    KEYWORDS_RUST.contains(&keyword)
}

// ---------------------------------------------------------------------------

struct StructInfo {
    domain: String,
    module: String,
    version: Option<(u8, u8, u8)>,
    name: String,
    token: TokenStream,
    use_serde: bool,
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

// ---------------------------------------------------------------------------

struct PropertyInfo {
    name: String,
    token: TokenStream,
}

// ---------------------------------------------------------------------------

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
