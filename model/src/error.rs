#[derive(Debug)]
pub enum Error {
    Arg(String),
    InvalidComponentPath(String),
    InvalidUri(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFoundSchema(String),
    NotSupported(String),
    Yaml(serde_yaml::Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Error::Yaml(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl Error {
    pub fn arg(msg: &str) -> Self {
        Error::Arg(msg.to_string())
    }

    pub fn invalid_component_path(msg: &str) -> Self {
        Error::InvalidComponentPath(msg.to_string())
    }

    pub fn invalid_uri(msg: &str) -> Self {
        Error::InvalidUri(msg.to_string())
    }

    pub fn not_found_schema(msg: &str) -> Self {
        Error::NotFoundSchema(msg.to_string())
    }
}
