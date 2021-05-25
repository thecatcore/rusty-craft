use crate::minecraft_launcher::manifest::version::{Library, Main, VersionType};
use crate::minecraft_launcher::modding::ModLoaderInstaller;
use crate::minecraft_launcher::path;
use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use serde_json::Error;
use std::collections::HashMap;
use std::str::FromStr;

const MC_VERSIONS: [&str; 1] = ["b1.7.3"];
const LOADER_VERSIONS: &str =
    "https://storage.googleapis.com/devan-maven/io/github/minecraft-cursed-legacy/cursed-fabric-loader/maven-metadata.xml";
const PROFILE_NAMING: &str = "cursed-fabric-loader-{loader_version}-{mc_version}";
const LIBS: [(&str, &str); 16] = [
    (
        "net.fabricmc:tiny-mappings-parser:0.2.2.14",
        "https://maven.fabricmc.net/",
    ),
    (
        "net.fabricmc:sponge-mixin:0.8+build.18",
        "https://maven.fabricmc.net/",
    ),
    (
        "net.fabricmc:tiny-remapper:0.3.0.70",
        "https://maven.fabricmc.net/",
    ),
    (
        "net.fabricmc:fabric-loader-sat4j:2.3.5.4",
        "https://maven.fabricmc.net/",
    ),
    ("com.google.jimfs:jimfs:1.1", "https://maven.fabricmc.net/"),
    ("org.ow2.asm:asm:9.0", "https://maven.fabricmc.net/"),
    (
        "org.ow2.asm:asm-analysis:9.0",
        "https://maven.fabricmc.net/",
    ),
    ("org.ow2.asm:asm-commons:9.0", "https://maven.fabricmc.net/"),
    ("org.ow2.asm:asm-tree:9.0", "https://maven.fabricmc.net/"),
    ("org.ow2.asm:asm-util:9.0", "https://maven.fabricmc.net/"),
    ("org.apache.logging.log4j:log4j-api:2.8.1", ""),
    ("org.apache.logging.log4j:log4j-core:2.8.1", ""),
    (
        "com.google.code.gson:gson:2.8.6",
        "https://repo1.maven.org/maven2/",
    ),
    (
        "net.fabricmc:access-widener:1.0.0",
        "https://maven.fabricmc.net/",
    ),
    ("com.google.guava:guava:21.0", "https://maven.fabricmc.net/"),
    (
        "com.github.minecraft-cursed-legacy:Plasma:b1.7.3-build.19",
        "https://storage.googleapis.com/devan-maven/",
    ),
];
const LIB_NAME: &str = "com.github.minecraft-cursed-legacy:cursed-fabric-loader:{sha}";

#[derive(Clone)]
pub struct CursedLegacyInstaller {}

impl ModLoaderInstaller for CursedLegacyInstaller {
    fn get_name(&self) -> String {
        "Cursed Legacy".to_string()
    }

    fn get_compatible_versions(&self) -> Vec<String> {
        let mut versions = vec![];

        for version in MC_VERSIONS.iter() {
            versions.push(version.to_string());
        }

        versions
    }

    fn get_loader_versions(&self, mc_version: String) -> HashMap<String, String> {
        let mut map = HashMap::new();

        match path::read_file_from_url_to_string(LOADER_VERSIONS) {
            Ok(commit_list) => match deserialize_commit_list(commit_list) {
                Ok(commit_list) => {
                    for commit in commit_list {
                        let date = commit.commit.author.date.to_string();
                        let mut name = String::new();

                        let mut counter = 0;

                        for char in commit.sha.chars() {
                            if counter > 6 {
                                break;
                            }

                            name.push(char);

                            counter += 1;
                        }

                        map.insert(name, date);
                    }
                }
                Err(_) => {}
            },
            Err(err) => {}
        };

        map
    }

    fn save_compatible_loader_versions(&self) -> bool {
        true
    }

    fn get_profile_name_for_mc_version(&self, mc_version: String) -> String {
        PROFILE_NAMING.replace("{mc_version}", mc_version.as_str())
    }

    fn get_profile_name_for_loader_version(
        &self,
        mc_version: String,
        loader_version: String,
    ) -> String {
        self.get_profile_name_for_mc_version(mc_version)
            .replace("${loader_version}", loader_version.as_str())
    }

    fn create_profile(&self, mc_version: String, loader_version: String) -> Main {
        let id =
            self.get_profile_name_for_loader_version(mc_version.clone(), loader_version.clone());
        let loaders = self.get_loader_versions(mc_version.clone());
        let date = loaders.get(loader_version.clone().as_str()).unwrap();

        let mut libs = vec![];

        for lib in LIBS.iter() {
            libs.push(Library {
                downloads: None,
                name: lib.0.to_string(),
                extract: None,
                natives: None,
                rules: None,
                url: if lib.1.is_empty() {
                    None
                } else {
                    Some(lib.1.to_string())
                },
            });
        }

        libs.push(Library {
            downloads: None,
            name: LIB_NAME.replace("{sha}", loader_version.as_str()),
            extract: None,
            natives: None,
            rules: None,
            url: Some("https://jitpack.io/".to_string()),
        });

        Main {
            arguments: None,
            asset_index: None,
            assets: None,
            compliance_level: None,
            downloads: None,
            id,
            java_version: None,
            libraries: libs,
            logging: None,
            main_class: "net.fabricmc.loader.launch.knot.KnotClient".to_string(),
            minimum_launcher_version: None,
            release_time: DateTime::from_str(date).unwrap(),
            time: DateTime::from_str(date).unwrap(),
            _type: VersionType::OldBeta,
            minecraft_arguments: None,
            inherits_from: Some(mc_version),
        }
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        Box::new(CursedLegacyInstaller {})
    }
}

fn deserialize_commit_list(commit_list_raw: String) -> serde_json::Result<Vec<Commit>> {
    let commit_list = serde_json::from_str(commit_list_raw.as_str());

    commit_list
}

#[derive(Deserialize, Clone)]
struct Commit {
    pub sha: String,
    pub commit: CommitInfo,
}

#[derive(Deserialize, Clone)]
struct CommitInfo {
    pub author: CommitAuthor,
}

#[derive(Deserialize, Clone)]
struct CommitAuthor {
    pub date: DateTime<Utc>,
}
