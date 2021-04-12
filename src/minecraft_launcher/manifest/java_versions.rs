use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use std::env::consts;

#[derive(Deserialize, Clone)]
pub struct Main {
    pub linux: OsVersions,
    #[serde(alias = "linux-i386")]
    pub linux_i386: OsVersions,
    #[serde(alias = "mac-os")]
    pub mac_os: OsVersions,
    #[serde(alias = "windows-x64")]
    pub windows_x64: OsVersions,
    #[serde(alias = "windows-x86")]
    pub windows_x86: OsVersions,
}

impl Main {
    pub fn get_os_version(self) -> Option<OsVersions> {
        match consts::OS {
            "windows" => match consts::ARCH {
                "x86" => Some(self.windows_x86),    //"windows-x86",
                "x86_64" => Some(self.windows_x64), //"windows-x64",
                &_ => None,
            },
            "macos" => Some(self.mac_os), //"mac-os",
            &_ => match consts::ARCH {
                "x86" => Some(self.linux_i386), //"linux-i386",
                &_ => Some(self.linux),         //"linux"
            },
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct OsVersions {
    #[serde(alias = "java-runtime-alpha")]
    pub java_runtime_alpha: Vec<Version>,
    #[serde(alias = "jre-legacy")]
    pub jre_legacy: Vec<Version>,
    #[serde(alias = "minecraft-java-exe")]
    pub java_exe: Vec<Version>,
}

impl OsVersions {
    pub fn get_java_version(self, version: &String) -> Option<Vec<Version>> {
        match version.as_str() {
            "java-runtime-alpha" => Some(self.java_runtime_alpha),
            "jre-legacy" => Some(self.jre_legacy),
            "minecraft-java-exe" => Some(self.java_exe),
            &_ => None,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Version {
    pub availability: Availability,
    pub manifest: Manifest,
    pub version: VersionInfo,
}

#[derive(Deserialize, Clone)]
pub struct Availability {
    pub group: u64,
    pub progress: u64,
}

#[derive(Deserialize, Clone)]
pub struct Manifest {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub released: DateTime<Utc>,
}

pub fn parse_java_versions_manifest(version_str: &String) -> serde_json::Result<Main> {
    let version_test: serde_json::Result<Main> = serde_json::from_str(version_str);

    version_test
}
