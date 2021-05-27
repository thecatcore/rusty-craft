use chrono::{DateTime, Utc};
use reqwest::blocking::get as get_url;
use serde_derive::Deserialize;
use serde_xml_rs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_body_from_url_else_from_file(url: &str, path: &Path) -> Option<String> {
    match get_url(url) {
        Ok(mut response) => {
            let mut manifest_body = String::new();
            match response.read_to_string(&mut manifest_body) {
                Ok(_) => Option::Some(manifest_body),
                Err(_) => Option::None,
            }
        }
        Err(_) => {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(_) => {
                    return Option::None;
                }
            };

            let mut body = String::new();
            match file.read_to_string(&mut body) {
                Ok(_) => Option::Some(body),
                Err(_) => Option::None,
            }
        }
    }
}

#[derive(Deserialize, PartialEq)]
pub struct MavenMetadata {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub versioning: MavenVersioning,
}

impl MavenMetadata {
    pub fn from_str(string: &str) -> Result<MavenMetadata, String> {
        let metadata: MavenMetadata = match serde_xml_rs::from_str(string) {
            Ok(meta) => meta,
            Err(err) => return Err(err.to_string()),
        };

        Ok(metadata)
    }
}

#[derive(Deserialize, PartialEq)]
pub struct MavenVersioning {
    pub release: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
    pub versions: MavenVersion,
}

#[derive(Deserialize, PartialEq)]
pub struct MavenVersion {
    pub version: Vec<MavVersion>,
}

#[derive(Deserialize, PartialEq)]
pub struct MavVersion {
    #[serde(rename = "$value")]
    pub body: String,
}
