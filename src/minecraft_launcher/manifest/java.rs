use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Main {
    pub files: HashMap<String, Element>,
}

#[derive(Deserialize, Clone)]
pub struct Element {
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(default)]
    pub executable: bool,
    pub downloads: Option<Downloads>,
    pub target: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Downloads {
    pub lzma: Option<FileInfo>,
    pub raw: FileInfo,
}

#[derive(Deserialize, Clone)]
pub struct FileInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

pub fn parse_java_version_manifest(version_str: &String) -> serde_json::Result<Main> {
    let version_test: serde_json::Result<Main> = serde_json::from_str(version_str);

    version_test
}
