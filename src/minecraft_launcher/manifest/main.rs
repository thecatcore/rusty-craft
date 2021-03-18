use serde;
use serde::Deserialize;
use serde_derive::Deserialize;

use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Main {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize)]
pub struct Version {
    pub id: String,
    #[serde(alias = "type")]
    pub _type: String,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(alias = "releaseTime")]
    pub release_time: DateTime<Utc>,
    pub sha1: String,
    #[serde(alias = "complianceLevel")]
    pub compliance_level: u8,
}

pub fn parse_manifest(manifest_str: &String, version_folder: &PathBuf) -> serde_json::Result<Main> {
    let manifest: serde_json::Result<Main> = serde_json::from_str(manifest_str);

    manifest
}
