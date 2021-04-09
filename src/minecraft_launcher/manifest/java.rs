use std::collections::HashMap;
use serde;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Main {
    pub files: HashMap<String, Element>
}

#[derive(Deserialize, Clone)]
pub struct Element {
    #[serde(alias = "type")]
    pub element_type: String,
    pub executable: Option<bool>,
    pub downloads: Option<Downloads>
}

#[derive(Deserialize, Clone)]
pub struct Downloads {
    pub lzma: FileInfo,
    pub raw: FileInfo
}

#[derive(Deserialize, Clone)]
pub struct FileInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String
}

pub fn parse_java_version_manifest(version_str: &String) -> serde_json::Result<Main> {
    let version_test: serde_json::Result<Main> = serde_json::from_str(version_str);

    version_test
}