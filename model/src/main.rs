mod error;
mod gen_code;

use self::error::Error;
use self::gen_code::gen_code;
use openapi_spec_schema::{
    OpenApi, Operation, PartOpenApi, ReferenceOr, RequestBody, Response, Schema, SchemaType,
    SchemaTypes,
};
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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

    check_schema(&mut schemas);

    println!("Source Code Generating...");
    gen_code(&output, &schemas)?;

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

/// 再帰的にディレクトリを探索して openapi.yaml ファイルからスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `dir` - 探索対象ディレクトリ
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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

/// `entry_file` のスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - 対象ファイルパス
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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

/// `op` のスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - 対象ファイルパス
/// * `path` - API の URI
/// * `method` - API の HTTP メソッド
/// * `op` - API の操作
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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

/// リクエストのスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - 対象ファイルパス
/// * `path` - API の URI
/// * `method` - API の HTTP メソッド
/// * `req` - リクエスト
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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
            let schema_name = format!("{}-{}-Request", method.to_lowercase(), resource_name(path));
            collect_anonymous_or_child(
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

    Ok(())
}

/// レスポンスのスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - 対象ファイルパス
/// * `status` - API の URI, API のメソッド, HTTP ステータスコード
/// * `res` - レスポンス
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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
                let schema_name = format!(
                    "{}-{}-{}Response",
                    status.1.to_lowercase(),
                    resource_name(status.0),
                    status.2
                );
                collect_anonymous_or_child(
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

    Ok(())
}

/// スキーマを特定型として収集か、中にあるスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - スキーマを定義しているファイルパス
/// * `schema` - スキーマ
/// * `schema_name` - スキーマの名前
/// * `another_file` - バージョンが異なるファイルを読み込むかどうか
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
fn collect_anonymous_or_child(
    root: &Path,
    entry_file: &Path,
    schema: &Schema,
    schema_name: &str,
    another_file: bool,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if anonymous_ty(schema) {
        collect_anonymous(
            root,
            entry_file,
            schema,
            schema_name,
            another_file,
            scaned_files,
            schemas,
        )?;
    } else {
        collect_schema_child(
            root,
            entry_file,
            schema,
            schema_name,
            another_file,
            scaned_files,
            schemas,
        )?;
    }

    Ok(())
}

/// スキーマを特定型として収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - スキーマを定義しているファイルパス
/// * `schema` - スキーマ
/// * `schema_name` - スキーマの名前
/// * `another_file` - バージョンが異なるファイルを読み込むかどうか
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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
        anony: true,
        duplicated: false,
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

/// スキーマの中にあるスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - スキーマを定義しているファイルパス
/// * `parent` - スキーマ
/// * `parent_name` - スキーマの名前
/// * `another_file` - バージョンが異なるファイルを読み込むかどうか
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
fn collect_schema_child(
    root: &Path,
    entry_file: &Path,
    parent: &Schema,
    parent_name: &str,
    another_file: bool,
    scaned_files: &mut Vec<PathBuf>,
    schemas: &mut Vec<SchemaItem>,
) -> Result<(), Error> {
    if let Some(r) = parent.r#ref.as_ref() {
        collect_reference(root, entry_file, r, another_file, scaned_files, schemas)?;
    }

    if let Some(i) = parent.items.as_ref() {
        collect_anonymous_or_child(
            root,
            entry_file,
            i,
            parent_name,
            another_file,
            scaned_files,
            schemas,
        )?;
    }

    if let Some(any_of) = parent.any_of.as_ref() {
        for (i, s) in any_of.iter().enumerate() {
            let schema_name = format!("{}-{}", parent_name, i);
            collect_anonymous_or_child(
                root,
                entry_file,
                s,
                &schema_name,
                another_file,
                scaned_files,
                schemas,
            )?;
        }
    }

    if let Some(one_of) = parent.one_of.as_ref() {
        for (i, s) in one_of.iter().enumerate() {
            let schema_name = format!("{}-{}", parent_name, i);
            collect_anonymous_or_child(
                root,
                entry_file,
                s,
                &schema_name,
                another_file,
                scaned_files,
                schemas,
            )?;
        }
    }

    collect_schema_property(
        root,
        entry_file,
        parent,
        parent_name,
        another_file,
        scaned_files,
        schemas,
    )?;

    Ok(())
}

/// 参照先のスキーマを収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - `r` がフラグメントのみの場合のファイルパス
/// * `r` - 参照
/// * `another_file` - バージョンが異なるファイルを読み込むかどうか
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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
                            anony: false,
                            duplicated: false,
                        };

                        schemas.push(item);

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

/// スキーマの中にある `properties` を収集する。
///
/// * `root` - ルートディレクトリ
/// * `entry_file` - スキーマを定義しているファイルパス
/// * `entry` - スキーマ
/// * `entry_name` - スキーマの名前
/// * `another_file` - バージョンが異なるファイルを読み込むかどうか
/// * `scaned_files` - 収集済みファイルパス
/// * `schemas` - 収集したスキーマ
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
            let schema_name = format!("{entry_name}-{prop_name}");
            collect_anonymous_or_child(
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

    Ok(())
}

/// 匿名型かどうかを判定する。
fn anonymous_ty(schema: &Schema) -> bool {
    (schema.r#type == Some(SchemaTypes::Unit(SchemaType::Object)) && schema.r#ref.is_none())
        || (schema.r#enum.is_some() && schema.r#ref.is_none())
        || schema
            .any_of
            .as_ref()
            .map(|s| s.iter().len() > 1)
            .unwrap_or_default()
        || schema
            .one_of
            .as_ref()
            .map(|s| s.iter().len() > 1)
            .unwrap_or_default()
}

/// 生成される Rust 型が他のスキーマと重複する匿名型のスキーマをマーキングする。
fn check_schema(schemas: &mut Vec<SchemaItem>) {
    let copied = schemas.clone();
    let config = Config { schemas: &copied };

    let copied = copied
        .iter()
        .map(|s| {
            let ty_names = config.ref_types_name(&s.domain_name, &s.schema_name, false);
            ty_names.join("::")
        })
        .collect::<Vec<String>>();

    for schema in schemas {
        let ty_names = config.ref_types_name(&schema.domain_name, &schema.schema_name, false);
        let ty_name = ty_names.join("::");

        if schema.anony && copied.iter().filter(|&s| s == &ty_name).count() > 1 {
            schema.duplicated = true;
            println!("Warning: Duplicate {}.", schema.schema_name);
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SchemaItem {
    /// ドメイン名
    domain_name: String,
    /// ファイル名
    schema_file_name: String,
    /// フラグメント
    schema_name: String,
    /// スキーマ
    schema: Schema,
    /// 匿名型かどうか
    anony: bool,
    /// 構造体の名前が重複するかどうか
    duplicated: bool,
}

impl SchemaItem {
    fn r#ref(&self) -> String {
        let domain_name = &self.domain_name;
        let schema_file_name = &self.schema_file_name;
        let schema_name = &self.schema_name;
        format!("{domain_name}/{schema_file_name}#{schema_name}")
    }
}

// ---------------------------------------------------------------------------

pub struct Config<'a> {
    schemas: &'a [SchemaItem],
}

impl<'a> Config<'a> {
    /// 指定した名前のスキーマを取得する。
    fn get_def_by_name(&self, domain: &str, name: &str) -> Result<&'a SchemaItem, Error> {
        self.schemas
            .iter()
            .find(|&s| s.domain_name == domain && s.schema_name == name)
            .ok_or(Error::not_found_schema(name))
    }

    /// 指定した参照のスキーマを取得する。
    fn get_def_by_ref(&self, r: &str) -> Result<&'a SchemaItem, Error> {
        self.schemas
            .iter()
            .find(|&s| s.r#ref() == r)
            .ok_or(Error::not_found_schema(r))
    }

    /// 指定した URL のスキーマを取得する。
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

    /// ドメイン名を取得する。
    ///
    /// `http://redfish.dmtf.org/schemas/domain/v1/` の形式から
    /// `domain` を返却する。存在しない場合は空文字を返却する。
    fn domain(&self, url: &str, domain_name: &str) -> String {
        let domain_name = url.split('/').nth(4).unwrap_or(domain_name);
        if domain_name == "v1" {
            "".to_string()
        } else {
            domain_name.to_string()
        }
    }

    /// スキーマの名前からモジュールの名前とバージョンを取得する。
    ///
    /// `module_vX_Y_Z_name` の形式から `module` と `(X, Y, X)` を返却する。
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

    /// スキーマの名前から型名を取得する。
    ///
    /// `module_vX_Y_Z_name` の形式から `name` を返却する。
    fn types_name(&self, source: &str, duplicated: bool) -> String {
        let (_, value) = source.rsplit_once('_').unwrap_or(("", source));

        let mut v = value.to_string();
        if duplicated {
            v = format!("{}-Anony", v);
        }

        upper_camel_case(&v)
    }

    /// 列挙型のフィールドの名前を取得する。
    fn enum_variants_name(&self, source: &str) -> String {
        upper_camel_case(source)
    }

    /// 構造体のフィールドの名前を取得する。
    fn fields_name(&self, source: &str) -> String {
        snake_case(source)
    }

    /// 指定したスキーマの名前から型の階層を取得する。
    fn ref_types_name(&self, domain: &str, source: &str, duplicated: bool) -> Vec<String> {
        let (module, version) = self.modules_name(source);
        let ty_name = self.types_name(source, duplicated);

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

        type_names
    }
}

// ---------------------------------------------------------------------------

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

/// ドメイン名を取得する。
///
/// `http://redfish.dmtf.org/schemas/domain/v1/` の形式から
/// `domain` を返却する。存在しない場合は空文字を返却する。
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

/// URI をもとに '-' で連結したリソース名を取得する。
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
