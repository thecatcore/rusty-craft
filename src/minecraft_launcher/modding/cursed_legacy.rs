use crate::minecraft_launcher::manifest::version::{Library, Main, VersionType};
use crate::minecraft_launcher::modding::ModLoaderInstaller;
use crate::minecraft_launcher::path;
use chrono::{DateTime, Utc};
use crate::minecraft_launcher::utils;
use std::collections::HashMap;
use std::str::FromStr;
use crate::minecraft_launcher::utils::MavenMetadata;

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
        "io.github.minecraft-cursed-legacy:Plasma:b1.7.3-build.19",
        "https://storage.googleapis.com/devan-maven/",
    ),
];
const LIB_NAME: &str = "io.github.minecraft-cursed-legacy:cursed-fabric-loader:{version}";

#[derive(Clone)]
pub struct CursedLegacyInstaller {}

impl ModLoaderInstaller for CursedLegacyInstaller {
    fn get_name(&self) -> String {
        "Cursed Legacy".to_string()
    }

    fn get_compatible_versions(&self) -> Result<Vec<String>, String> {
        let mut versions = vec![];

        for version in MC_VERSIONS.iter() {
            versions.push(version.to_string());
        }

        Ok(versions)
    }

    fn get_loader_versions(&self, mc_version: String) -> Result<HashMap<String, String>, String> {
        let mut map = HashMap::new();

        let mut key_list = vec![];

        let raw_maven_metadata = path::read_file_from_url_to_string(LOADER_VERSIONS)?;
        let maven_metadata = utils::MavenMetadata::from_str(raw_maven_metadata.as_str())?;

        for version in maven_metadata.versioning.versions.version {
            if version.body.contains("local") {
                continue;
            }

            let mut date = "Unknown".to_string();
            if version.body == maven_metadata.versioning.release {
                date = maven_metadata.versioning.last_updated.to_string();
            }

            map.insert(version.body.clone(), date);
            key_list.insert(0, version.body);
        }

        let mut sorted_map = HashMap::new();

        for key in key_list {
            sorted_map.insert(key.clone(), map.get(&key).unwrap().clone());
        }

        Ok(sorted_map)
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

    fn create_profile(&self, mc_version: String, loader_version: String) -> Result<Main, String> {
        let id =
            self.get_profile_name_for_loader_version(mc_version.clone(), loader_version.clone());

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
            name: LIB_NAME.replace("{version}", loader_version.as_str()),
            extract: None,
            natives: None,
            rules: None,
            url: Some("https://storage.googleapis.com/devan-maven/".to_string()),
        });

        Ok(Main {
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
            release_time: Utc::now(),
            time: Utc::now(),
            _type: VersionType::OldBeta,
            minecraft_arguments: None,
            inherits_from: Some(mc_version),
        })
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        Box::new(CursedLegacyInstaller {})
    }
}
