use crate::minecraft_launcher::manifest::version::{Library, Main, VersionType};
use crate::minecraft_launcher::modding::ModLoaderInstaller;
use chrono::Utc;
use std::collections::HashMap;

const MAVEN: &str = "http://dl.liteloader.com/versions/";
const MAIN_CLASS: &str = "net.minecraft.launchwrapper.Launch";
const VERSIONS: [(&str, bool, [&str; 4], &str); 16] = [
    (
        "1.12.2",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.2",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ), // This one use http://repo.liteloader.com/.
    (
        "1.12.1",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.12",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.11.2",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.11",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.10.2",
        true,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.10",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.9.4",
        true,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.9",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.8.9",
        false,
        [
            "net.minecraft:launchwrapper:1.12",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userProperties ${user_properties} --userType ${user_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.8",
        true,
        [
            "net.minecraft:launchwrapper:1.11",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userProperties ${user_properties} --userType ${user_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.7.10",
        true,
        [
            "net.minecraft:launchwrapper:1.11",
            "",
            "org.ow2.asm:asm-all:5.0.3",
            "com.google.guava:guava:16.0"
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userProperties ${user_properties} --userType ${user_type} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker",
    ),
    (
        "1.7.2",
        true,
        [
            "net.minecraft:launchwrapper:1.9",
            "",
            "org.ow2.asm:asm-all:4.1",
            ""
        ],
        "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${game_assets} --uuid ${auth_uuid} --accessToken ${auth_access_token} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.6.4",
        true,
        [
            "net.minecraft:launchwrapper:1.8",
            "lzma:lzma:0.0.1",
            "",
            ""
        ],
        "--username ${auth_player_name} --session ${auth_session} --version ${version_name} --gameDir ${game_directory} --assetsDir ${game_assets} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.6.2",
        true,
        [
            "net.minecraft:launchwrapper:1.3",
            "lzma:lzma:0.0.1",
            "",
            ""
        ],
        "--username ${auth_player_name} --session ${auth_session} --gameDir ${game_directory} --assetsDir ${game_assets} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    ),
    (
        "1.5.2",
        true,
        [
            "",
            "",
            "",
            ""
        ],
        "${auth_player_name} ${auth_session} --gameDir ${game_directory} --assetsDir ${game_assets} --tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker"
    )
];
const LOADER_NAME: &str = "com.mumfrey:liteloader:{mc_version}";
const LOADER_NAME_SNAPSHOT: &str = "com.mumfrey:liteloader:{mc_version}-SNAPSHOT";
const NEW_PROFILE_NAMING: &str = "{mc_version}-LiteLoader{mc_version}";
// 1.5.2, 1.6.2 and 1.6.4
const OLD_PROFILE_NAMING: &str = "LiteLoader{mc_version}";

#[derive(Clone)]
pub struct LiteLoaderInstaller {}

impl LiteLoaderInstaller {
    pub fn new() -> LiteLoaderInstaller {
        LiteLoaderInstaller {}
    }

    fn get_version(
        &self,
        mc_version: String,
    ) -> Option<(&'static str, bool, [&'static str; 4], &'static str)> {
        for version in VERSIONS.iter() {
            if version.0 == mc_version.as_str() {
                return Some(*version);
            }
        }
        None
    }
}

impl ModLoaderInstaller for LiteLoaderInstaller {
    fn get_name(&self) -> String {
        "LiteLoader".to_string()
    }

    fn get_compatible_versions(&self) -> Result<Vec<String>, String> {
        let mut versions = Vec::new();

        for version in VERSIONS.iter() {
            versions.push(version.0.to_string())
        }

        Ok(versions)
    }

    fn get_loader_versions(&self, _mc_version: String) -> Result<HashMap<String, String>, String> {
        let mut map = HashMap::new();
        map.insert("LiteLoader".to_string(), "Unknown".to_string());

        Ok(map)
    }

    fn get_profile_name_for_mc_version(&self, mc_version: String) -> String {
        let mc_version = mc_version.as_str();
        if mc_version == "1.5.2" || mc_version == "1.6.2" || mc_version == "1.6.4" {
            String::from(OLD_PROFILE_NAMING).replace("{mc_version}", mc_version)
        } else {
            String::from(NEW_PROFILE_NAMING).replace("{mc_version}", mc_version)
        }
    }

    fn get_profile_name_for_loader_version(
        &self,
        mc_version: String,
        _loader_version: String,
    ) -> String {
        self.get_profile_name_for_mc_version(mc_version)
    }

    fn create_profile(&self, mc_version: String, _loader_version: String) -> Result<Main, String> {
        let version_info = self.get_version(mc_version.clone()).expect(":flushed:");
        let id = self.get_profile_name_for_mc_version(mc_version.clone());
        let inherits_from = mc_version.clone();
        let mut libraries: Vec<Library> = Vec::new();
        // LiteLoader
        libraries.push(Library {
            downloads: None,
            name: if version_info.1 {
                String::from(LOADER_NAME).replace("{mc_version}", &*mc_version)
            } else {
                String::from(LOADER_NAME_SNAPSHOT).replace("{mc_version}", &*mc_version)
            },
            extract: None,
            natives: None,
            rules: None,
            url: Some(String::from(MAVEN)),
        });
        // Launch Wrapper
        if !version_info.2[0].is_empty() {
            libraries.push(Library {
                downloads: None,
                name: version_info.2[0].to_string(),
                extract: None,
                natives: None,
                rules: None,
                url: None,
            });
        }
        // lzma
        if !version_info.2[1].is_empty() {
            libraries.push(Library {
                downloads: None,
                name: version_info.2[1].to_string(),
                extract: None,
                natives: None,
                rules: None,
                url: None,
            });
        }
        // ASM
        if !version_info.2[2].is_empty() {
            libraries.push(Library {
                downloads: None,
                name: version_info.2[2].to_string(),
                extract: None,
                natives: None,
                rules: None,
                url: if mc_version.as_str() == "1.12.2" {
                    Some(String::from("http://repo.liteloader.com/"))
                } else {
                    None
                },
            });
        }
        // guava
        if !version_info.2[3].is_empty() {
            libraries.push(Library {
                downloads: None,
                name: version_info.2[3].to_string(),
                extract: None,
                natives: None,
                rules: None,
                url: None,
            });
        }

        Ok(Main {
            arguments: None,
            asset_index: None,
            assets: None,
            compliance_level: None,
            downloads: None,
            id,
            java_version: None,
            libraries,
            logging: None,
            main_class: MAIN_CLASS.to_string(),
            minimum_launcher_version: None,
            release_time: Utc::now(),
            time: Utc::now(),
            _type: VersionType::Release,
            minecraft_arguments: Some(String::from(version_info.3)),
            inherits_from: Some(inherits_from),
        })
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        Box::new(LiteLoaderInstaller::new())
    }
}
