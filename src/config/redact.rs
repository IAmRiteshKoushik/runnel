use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Redaction {
    pub path: Vec<String>,
}
