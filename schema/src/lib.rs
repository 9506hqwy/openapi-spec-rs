pub mod model;

// https://spec.openapis.org/oas/v3.0.3
// https://spec.openapis.org/oas/v3.1.0

use self::model::{Any, Extensions, HttpStatuses, KeyValues};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BooleanOr<T> {
    Value(T),
    Boolean(bool),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ReferenceOr<T> {
    Value(T),
    Ref(Reference),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OpenApi {
    pub openapi: String,

    pub info: Info,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none", rename = "jsonSchemaDialect")]
    pub json_schema_dialect: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    // required until v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Paths>,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<HashMap<String, ReferenceOr<PathItem>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentation>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Info {
    pub title: String,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "termsOfService")]
    pub terms_of_service: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,

    pub version: String,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum License {
    Url(LicenseUrl),
    // since v3.1.0
    Id(LicenseId),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LicenseId {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LicenseUrl {
    pub name: String,

    pub url: String,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Server {
    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, ServerVariable>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ServerVariable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,

    pub default: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Components {
    // or reference until v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<HashMap<String, ReferenceOr<Response>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, ReferenceOr<Parameter>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, ReferenceOr<Example>>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "requestBodies")]
    pub request_bodies: Option<HashMap<String, ReferenceOr<RequestBody>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, ReferenceOr<Header>>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "securitySchemes")]
    pub security_schemes: Option<HashMap<String, ReferenceOr<SecurityScheme>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, ReferenceOr<Link>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<HashMap<String, ReferenceOr<Callback>>>,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none", rename = "pathItems")]
    pub path_items: Option<HashMap<String, ReferenceOr<PathItem>>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

type Paths = KeyValues<PathItem>;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none", rename = "$ref")]
    pub r#ref: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ReferenceOr<Parameter>>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentation>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "operationId")]
    pub operation_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ReferenceOr<Parameter>>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "requestBody")]
    pub request_body: Option<ReferenceOr<RequestBody>>,

    // required until v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<Responses>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<HashMap<String, ReferenceOr<Callback>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ExternalDocumentation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub url: String,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Parameter {
    pub name: String,

    pub r#in: ParameterIn,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowEmptyValue")]
    pub allow_empty_value: Option<bool>,

    #[serde(flatten)]
    pub pattern: ParameterPattern,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ParameterPattern {
    Content(ParameterContent),
    Style(ParameterStyle),
}

impl Default for ParameterPattern {
    fn default() -> Self {
        ParameterPattern::Style(ParameterStyle::default())
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ParameterStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    explode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowReserved")]
    allow_reserved: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<Schema>,

    #[serde(flatten)]
    example: Examples,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Examples {
    Values {
        examples: HashMap<String, ReferenceOr<Example>>,
    },
    Value {
        #[serde(skip_serializing_if = "Option::is_none")]
        example: Option<Any>,
    },
}

impl Default for Examples {
    fn default() -> Self {
        Examples::Value { example: None }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ParameterContent {
    content: HashMap<String, MediaType>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum ParameterIn {
    #[default]
    #[serde(rename = "query")]
    Query,

    #[serde(rename = "header")]
    Header,

    #[serde(rename = "path")]
    Path,

    #[serde(rename = "cookie")]
    Cookie,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub content: HashMap<String, MediaType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MediaType {
    // or reference until v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,

    #[serde(flatten)]
    example: Examples,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<HashMap<String, Encoding>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Encoding {
    #[serde(skip_serializing_if = "Option::is_none", rename = "contentType")]
    pub content_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, ReferenceOr<Header>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowReserved")]
    pub allow_reserved: Option<bool>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Responses {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#default: Option<ReferenceOr<Response>>,

    #[serde(flatten)]
    pub statuses: HttpStatuses<ReferenceOr<Response>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Response {
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, ReferenceOr<Header>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, ReferenceOr<Link>>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

type Callback = KeyValues<ReferenceOr<PathItem>>;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Example {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(flatten)]
    pub value: ExampleValue,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ExampleValue {
    Url {
        #[serde(rename = "externalValue")]
        external_value: String,
    },
    Literal {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Any>,
    },
}

impl Default for ExampleValue {
    fn default() -> Self {
        ExampleValue::Literal { value: None }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Link {
    #[serde(flatten)]
    pub operation: LinkOperation,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Any>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "requestBody")]
    pub request_body: Option<Any>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LinkOperation {
    #[serde(rename = "operationRef")]
    Ref(String),

    #[serde(rename = "operationId")]
    Id(String),
}

impl Default for LinkOperation {
    fn default() -> Self {
        LinkOperation::Id("".to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowEmptyValue")]
    pub allow_empty_value: Option<bool>,

    #[serde(flatten)]
    pub pattern: ParameterPattern,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Tag {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub descrption: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentation>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub r#ref: String,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Schema {
    // until v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Any>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00#section-8.2.3.1
    #[serde(skip_serializing_if = "Option::is_none", rename = "$ref")]
    pub r#ref: Option<String>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00#section-10.2.1
    #[serde(skip_serializing_if = "Option::is_none", rename = "allOf")]
    pub all_of: Option<Vec<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "anyOf")]
    pub any_of: Option<Vec<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "oneOf")]
    pub one_of: Option<Vec<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Schema>>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00#section-10.2.2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#if: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub then: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#else: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "dependentSchemas")]
    pub dependent_schemas: Option<Any>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00#section-10.3.1
    #[serde(skip_serializing_if = "Option::is_none", rename = "prefixItems")]
    pub prefix_items: Option<Vec<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<Schema>>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00#section-10.3.2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "patternProperties")]
    pub pattern_properties: Option<HashMap<String, Schema>>,

    // bool support until v3.1.0
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "additionalProperties"
    )]
    pub additional_properties: Option<Box<BooleanOr<Schema>>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "propertyNames")]
    pub property_names: Option<Box<Schema>>,

    // unit only until v3.1.0
    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-6.1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<SchemaTypes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#const: Option<Any>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-6.2
    #[serde(skip_serializing_if = "Option::is_none", rename = "multipleOf")]
    pub multiple_of: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<i32>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-6.3
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxLength")]
    pub max_length: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "minLength")]
    pub min_length: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-6.4
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxItems")]
    pub max_items: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "minItems")]
    pub min_items: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "uniqueItems")]
    pub unique_items: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "maxContains")]
    pub max_contains: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "minContains")]
    pub min_contains: Option<u32>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-6.5
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxProperties")]
    pub max_properties: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "minProperties")]
    pub min_properties: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "dependentRequired")]
    pub dependent_required: Option<Any>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-8
    #[serde(skip_serializing_if = "Option::is_none", rename = "contentEncoding")]
    pub content_encoding: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "contentMediaType")]
    pub content_media_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "contentSchema")]
    pub content_schema: Option<Box<Schema>>,

    // https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-9
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Any>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "readOnly")]
    pub read_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "writeOnly")]
    pub write_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<Any>>,
    // TODO: add more `JSON Schema` support.
    #[serde(flatten)]
    pub extensions: KeyValues<Any>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum SchemaType {
    #[default]
    #[serde(rename = "null")]
    Null,

    #[serde(rename = "boolean")]
    Boolean,

    #[serde(rename = "object")]
    Object,

    #[serde(rename = "array")]
    Array,

    #[serde(rename = "number")]
    Number,

    #[serde(rename = "string")]
    String,

    #[serde(rename = "integer")]
    Integer,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SchemaTypes {
    Unit(SchemaType),
    Array(Vec<SchemaType>),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Discriminator {
    #[serde(rename = "propertyName")]
    pub property_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<HashMap<String, String>>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Xml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapped: Option<bool>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey(SecuritySchemeApiKey),

    #[serde(rename = "http")]
    Http(SecuritySchemeHttp),

    #[serde(rename = "oauth2")]
    Oauth2(SecuritySchemeOauth2),

    #[serde(rename = "openIdConnect")]
    OpenIdConnect(SecuritySchemeOpenIdConnect),

    // since v3.1.0
    #[serde(rename = "mutualTLS")]
    MutualTls(SecuritySchemeMutualTls),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SecuritySchemeApiKey {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Any>,

    pub name: String,

    pub r#in: SecuritySchemeIn,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SecuritySchemeHttp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Any>,

    pub scheme: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "bearerFormat")]
    pub bearer_format: Option<String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SecuritySchemeMutualTls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Any>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SecuritySchemeOauth2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Any>,

    pub flows: OAuthFlows,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SecuritySchemeOpenIdConnect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Any>,

    #[serde(rename = "openIdConnectUrl")]
    pub open_id_connect_url: String,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum SecuritySchemeIn {
    #[default]
    #[serde(rename = "query")]
    Query,

    #[serde(rename = "header")]
    Header,

    #[serde(rename = "cookie")]
    Cookie,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OAuthFlows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuthFlowImplicit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuthFlowPassword>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "clientCredentials")]
    pub client_credentials: Option<OAuthFlowClientCredentials>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "authorizationCode")]
    pub authorization_code: Option<OAuthFlowAuthorizationCode>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OAuthFlowImplicit {
    #[serde(rename = "authorizationUrl")]
    pub authrization_url: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "refreshUrl")]
    pub refresh_url: Option<String>,

    pub scopes: HashMap<String, String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OAuthFlowPassword {
    #[serde(rename = "tokenUrl")]
    pub token_url: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "refreshUrl")]
    pub refresh_url: Option<String>,

    pub scopes: HashMap<String, String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OAuthFlowClientCredentials {
    #[serde(rename = "tokenUrl")]
    pub token_url: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "refreshUrl")]
    pub refresh_url: Option<String>,

    pub scopes: HashMap<String, String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OAuthFlowAuthorizationCode {
    #[serde(rename = "authorizationUrl")]
    pub authrization_url: String,

    #[serde(rename = "tokenUrl")]
    pub token_url: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "refreshUrl")]
    pub refresh_url: Option<String>,

    pub scopes: HashMap<String, String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

type SecurityRequirement = HashMap<String, Vec<String>>;

// For splitting Redfish schema into multiple files.
// I do not understand this specification.
// I think that the part file is not compilicant to Schema object.
// https://spec.openapis.org/oas/v3.0.3#schemaObject
// > Additional properties defined by the JSON Schema specification that are not mentioned here are strictly unsupported.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PartOpenApi {
    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none", rename = "jsonSchemaDialect")]
    pub json_schema_dialect: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    // required until v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Paths>,

    // since v3.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<HashMap<String, ReferenceOr<PathItem>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentation>,

    #[serde(flatten)]
    pub extensions: KeyValues<Any>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_openapi() {
        let v = OpenApi::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"openapi\":\"\",\"info\":{\"title\":\"\",\"version\":\"\"}}",
            s
        );
        let r = serde_json::from_str::<OpenApi>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_openapi_extensions() {
        let mut v = OpenApi::default();
        v.extensions.values.insert("a".to_string(), Any::Number(0));
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"openapi\":\"\",\"info\":{\"title\":\"\",\"version\":\"\"},\"x-a\":0}",
            s
        );
        let r = serde_json::from_str::<OpenApi>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_openapi_paths_value() {
        let mut p = KeyValues::<PathItem>::default();
        p.values.insert("a".to_string(), PathItem::default());
        let v = OpenApi {
            paths: Some(p),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"openapi\":\"\",\"info\":{\"title\":\"\",\"version\":\"\"},\"paths\":{\"a\":{}}}",
            s
        );
        let r = serde_json::from_str::<OpenApi>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_openapi_paths_extension() {
        let mut p = KeyValues::<PathItem>::default();
        p.extensions.insert("a".to_string(), Any::Number(0));
        let v = OpenApi {
            paths: Some(p),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"openapi\":\"\",\"info\":{\"title\":\"\",\"version\":\"\"},\"paths\":{\"x-a\":0}}",
            s
        );
        let r = serde_json::from_str::<OpenApi>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_info() {
        let v = Info::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"title\":\"\",\"version\":\"\"}", s);
        let r = serde_json::from_str::<Info>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_contact() {
        let v = Contact::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Contact>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_license() {
        let v = License::Id(LicenseId::default());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\"}", s);
        let r = serde_json::from_str::<License>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_license_id() {
        let v = License::Id(LicenseId {
            identifier: Some("a".to_string()),
            ..Default::default()
        });
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\",\"identifier\":\"a\"}", s);
        let r = serde_json::from_str::<License>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_license_url() {
        let v = License::Url(LicenseUrl {
            url: "a".to_string(),
            ..Default::default()
        });
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\",\"url\":\"a\"}", s);
        let r = serde_json::from_str::<License>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_server() {
        let v = Server::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"url\":\"\"}", s);
        let r = serde_json::from_str::<Server>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_server_variable() {
        let v = ServerVariable::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"default\":\"\"}", s);
        let r = serde_json::from_str::<ServerVariable>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_components() {
        let v = Components::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Components>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_components_value() {
        let mut h = HashMap::new();
        h.insert(
            "a".to_string(),
            ReferenceOr::<Response>::Value(Response::default()),
        );
        let v = Components {
            responses: Some(h),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"responses\":{\"a\":{\"description\":\"\"}}}", s);
        let r = serde_json::from_str::<Components>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_components_ref() {
        let mut h = HashMap::new();
        h.insert(
            "a".to_string(),
            ReferenceOr::<Response>::Ref(Reference::default()),
        );
        let v = Components {
            responses: Some(h),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"responses\":{\"a\":{\"$ref\":\"\"}}}", s);
        let r = serde_json::from_str::<Components>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_path_item() {
        let v = PathItem::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<PathItem>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_operation() {
        let v = Operation::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Operation>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_operation_security() {
        let mut sec = HashMap::new();
        sec.insert("a".to_string(), vec![]);
        let v = Operation {
            security: Some(vec![sec]),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"security\":[{\"a\":[]}]}", s);
        let r = serde_json::from_str::<Operation>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_external_documentation() {
        let v = ExternalDocumentation::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"url\":\"\"}", s);
        let r = serde_json::from_str::<ExternalDocumentation>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_parameter() {
        let v = Parameter::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\",\"in\":\"query\"}", s);
        let r = serde_json::from_str::<Parameter>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_parameter_style() {
        let v = Parameter {
            pattern: ParameterPattern::Style(ParameterStyle {
                style: Some("a".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\",\"in\":\"query\",\"style\":\"a\"}", s);
        let r = serde_json::from_str::<Parameter>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_parameter_style_example() {
        let v = Parameter {
            pattern: ParameterPattern::Style(ParameterStyle {
                style: Some("a".to_string()),
                example: Examples::Value {
                    example: Some(Any::Number(0)),
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"name\":\"\",\"in\":\"query\",\"style\":\"a\",\"example\":0}",
            s
        );
        let r = serde_json::from_str::<Parameter>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_parameter_style_examples() {
        let v = Parameter {
            pattern: ParameterPattern::Style(ParameterStyle {
                style: Some("a".to_string()),
                example: Examples::Values {
                    examples: HashMap::new(),
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"name\":\"\",\"in\":\"query\",\"style\":\"a\",\"examples\":{}}",
            s
        );
        let r = serde_json::from_str::<Parameter>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_parameter_content() {
        let v = Parameter {
            pattern: ParameterPattern::Content(ParameterContent {
                content: HashMap::new(),
            }),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\",\"in\":\"query\",\"content\":{}}", s);
        let r = serde_json::from_str::<Parameter>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_request_body() {
        let v = RequestBody::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"content\":{}}", s);
        let r = serde_json::from_str::<RequestBody>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type() {
        let v = MediaType::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_example_boolean() {
        let v = MediaType {
            example: Examples::Value {
                example: Some(Any::Boolean(true)),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"example\":true}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_example_number() {
        let v = MediaType {
            example: Examples::Value {
                example: Some(Any::Number(0)),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"example\":0}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_example_string() {
        let v = MediaType {
            example: Examples::Value {
                example: Some(Any::String("a".to_string())),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"example\":\"a\"}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_example_array() {
        let v = MediaType {
            example: Examples::Value {
                example: Some(Any::Array(vec![Any::Number(0)])),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"example\":[0]}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_example_object() {
        let mut h = HashMap::new();
        h.insert("a".to_string(), Any::Number(0));
        let v = MediaType {
            example: Examples::Value {
                example: Some(Any::Object(h)),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"example\":{\"a\":0}}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_example_null() {
        let mut v = MediaType {
            example: Examples::Value {
                example: Some(Any::Null),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"example\":null}", s);
        // deserialize `null` to `None`.
        v.example = Examples::Value { example: None };
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_media_type_examples() {
        let v = MediaType {
            example: Examples::Values {
                examples: HashMap::new(),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"examples\":{}}", s);
        let r = serde_json::from_str::<MediaType>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_encoding() {
        let v = Encoding::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Encoding>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_responses() {
        let v = Responses::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Responses>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_responses_200() {
        let mut v = Responses::default();
        v.statuses
            .values
            .insert(200, ReferenceOr::<Response>::Value(Response::default()));
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"200\":{\"description\":\"\"}}", s);
        let r = serde_json::from_str::<Responses>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_responses_extension() {
        let mut v = Responses::default();
        v.extensions.values.insert("a".to_string(), Any::Number(0));
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"x-a\":0}", s);
        let r = serde_json::from_str::<Responses>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_response() {
        let v = Response::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"description\":\"\"}", s);
        let r = serde_json::from_str::<Response>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_example() {
        let v = Example::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Example>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_example_literal() {
        let v = Example {
            value: ExampleValue::Literal {
                value: Some(Any::Number(0)),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"value\":0}", s);
        let r = serde_json::from_str::<Example>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_example_url() {
        let v = Example {
            value: ExampleValue::Url {
                external_value: "a".to_string(),
            },
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"externalValue\":\"a\"}", s);
        let r = serde_json::from_str::<Example>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_link() {
        let v = Link::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"operationId\":\"\"}", s);
        let r = serde_json::from_str::<Link>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_link_operation_ref() {
        let v = Link {
            operation: LinkOperation::Ref("a".to_string()),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"operationRef\":\"a\"}", s);
        let r = serde_json::from_str::<Link>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_link_operation_id() {
        let v = Link {
            operation: LinkOperation::Id("a".to_string()),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"operationId\":\"a\"}", s);
        let r = serde_json::from_str::<Link>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_header() {
        let v = Header::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Header>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_tag() {
        let v = Tag::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"name\":\"\"}", s);
        let r = serde_json::from_str::<Tag>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_reference() {
        let v = Reference::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"$ref\":\"\"}", s);
        let r = serde_json::from_str::<Reference>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_schema() {
        let v = Schema::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Schema>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_schema_additional_properties_value() {
        let v = Schema {
            additional_properties: Some(Box::new(BooleanOr::Value(Schema::default()))),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"additionalProperties\":{}}", s);
        let r = serde_json::from_str::<Schema>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_schema_additional_properties_bool() {
        let v = Schema {
            additional_properties: Some(Box::new(BooleanOr::Boolean(true))),
            ..Default::default()
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"additionalProperties\":true}", s);
        let r = serde_json::from_str::<Schema>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_schema_extensions_value() {
        let mut v = Schema::default();
        v.extensions.values.insert("a".to_string(), Any::Number(0));
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"a\":0}", s);
        let r = serde_json::from_str::<Schema>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_schema_extensions_extension() {
        let mut v = Schema::default();
        v.extensions
            .extensions
            .insert("a".to_string(), Any::Number(0));
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"x-a\":0}", s);
        let r = serde_json::from_str::<Schema>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_discriminator() {
        let v = Discriminator::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"propertyName\":\"\"}", s);
        let r = serde_json::from_str::<Discriminator>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_xml() {
        let v = Xml::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<Xml>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_security_scheme_apikey() {
        let v = SecurityScheme::ApiKey(SecuritySchemeApiKey::default());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"type\":\"apiKey\",\"name\":\"\",\"in\":\"query\"}", s);
        let r = serde_json::from_str::<SecurityScheme>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_security_scheme_http() {
        let v = SecurityScheme::Http(SecuritySchemeHttp::default());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"type\":\"http\",\"scheme\":\"\"}", s);
        let r = serde_json::from_str::<SecurityScheme>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_security_scheme_mutualtls() {
        let v = SecurityScheme::MutualTls(SecuritySchemeMutualTls::default());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"type\":\"mutualTLS\"}", s);
        let r = serde_json::from_str::<SecurityScheme>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_security_scheme_oauth2() {
        let v = SecurityScheme::Oauth2(SecuritySchemeOauth2::default());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"type\":\"oauth2\",\"flows\":{}}", s);
        let r = serde_json::from_str::<SecurityScheme>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_security_scheme_openidconnect() {
        let v = SecurityScheme::OpenIdConnect(SecuritySchemeOpenIdConnect::default());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"type\":\"openIdConnect\",\"openIdConnectUrl\":\"\"}", s);
        let r = serde_json::from_str::<SecurityScheme>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_oauth_flows() {
        let v = OAuthFlows::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<OAuthFlows>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_oauth_flow_implicit() {
        let v = OAuthFlowImplicit::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"authorizationUrl\":\"\",\"scopes\":{}}", s);
        let r = serde_json::from_str::<OAuthFlowImplicit>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_oauth_flow_password() {
        let v = OAuthFlowPassword::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"tokenUrl\":\"\",\"scopes\":{}}", s);
        let r = serde_json::from_str::<OAuthFlowPassword>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_oauth_flow_client_credentials() {
        let v = OAuthFlowClientCredentials::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{\"tokenUrl\":\"\",\"scopes\":{}}", s);
        let r = serde_json::from_str::<OAuthFlowClientCredentials>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_oauth_flow_authorization_code() {
        let v = OAuthFlowAuthorizationCode::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"authorizationUrl\":\"\",\"tokenUrl\":\"\",\"scopes\":{}}",
            s
        );
        let r = serde_json::from_str::<OAuthFlowAuthorizationCode>(&s).unwrap();
        assert_eq!(v, r);
    }

    #[test]
    fn serde_part_openapi() {
        let v = PartOpenApi::default();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!("{}", s);
        let r = serde_json::from_str::<PartOpenApi>(&s).unwrap();
        assert_eq!(v, r);
    }
}
