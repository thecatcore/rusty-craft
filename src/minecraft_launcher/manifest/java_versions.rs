use chrono::{DateTime, Utc};
use serde;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Main {
    pub linux: OsVersions,
    #[serde(alias = "linux-i396")]
    pub linux_i396: OsVersions,
    #[serde(alias = "mac-os")]
    pub mac_os: OsVersions,
    #[serde(alias = "windows-x64")]
    pub windows_x64: OsVersions,
    #[serde(alias = "windows-x86")]
    pub windows_x86: OsVersions
}

#[derive(Deserialize, Clone)]
pub struct OsVersions {
    #[serde(alias = "java-runtime-alpha")]
    pub java_runtime_alpha: Vec<Version>,
    #[serde(alias = "jre-legacy")]
    pub jre_legacy: Vec<Version>,
    #[serde(alias = "java-exe")]
    pub java_exe: Vec<Version>
}

#[derive(Deserialize, Clone)]
pub struct Version {
    pub availability: Availability,
    pub manifest: Manifest,
    pub version: VersionInfo
}

#[derive(Deserialize, Clone)]
pub struct Availability {
    pub group: u64,
    pub progress: u64
}

#[derive(Deserialize, Clone)]
pub struct Manifest {
    pub sha1: String,
    pub size: u64,
    pub url: String
}

#[derive(Deserialize, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub released: DateTime<Utc>
}

pub fn parse_java_versions_manifest(version_str: &String) -> serde_json::Result<Main> {
    let version_test: serde_json::Result<Main> = serde_json::from_str(version_str);

    version_test
}