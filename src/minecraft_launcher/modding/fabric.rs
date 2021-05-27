use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::manifest::version::Main;
use crate::minecraft_launcher::modding::ModLoaderInstaller;
use crate::minecraft_launcher::path;
use serde_derive::Deserialize;
use serde_json::Error;
use std::collections::HashMap;

const MC_VERSIONS: &str = "https://meta.fabricmc.net/v2/versions/game";
const LOADER_VERSIONS: &str = "https://meta.fabricmc.net/v2/versions/loader/:game_version";
const JSON_PROFILE: &str =
    "https://meta.fabricmc.net/v2/versions/loader/:game_version/:loader_version/profile/json";
const PROFILE_NAMING: &str = "fabric-loader-{loader_version}-{mc_version}";

pub struct FabricInstaller {}

impl FabricInstaller {
    pub fn new() -> FabricInstaller {
        FabricInstaller {}
    }
}

impl ModLoaderInstaller for FabricInstaller {
    fn get_name(&self) -> String {
        "Fabric".to_string()
    }

    fn get_compatible_versions(&self) -> Result<Vec<String>, String> {
        let mut versions = vec![];

        let raw_version_list = path::read_file_from_url_to_string(MC_VERSIONS)?;
        let version_list = get_versions(raw_version_list.as_str())?;

        for version in version_list {
            versions.push(version.version);
        }

        Ok(versions)
    }

    fn save_compatible_versions(&self) -> bool {
        true
    }

    fn get_loader_versions(&self, mc_version: String) -> Result<HashMap<String, String>, String> {
        let mut versions = HashMap::new();

        let raw_version_list = path::read_file_from_url_to_string(
            LOADER_VERSIONS
                .replace(":game_version", mc_version.as_str())
                .as_str(),
        )?;
        let version_list = get_loader_versions(raw_version_list.as_str())?;

        for version in version_list {
            versions.insert(version.loader.version, "Unknown".to_string());
        }

        Ok(versions)
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
            .replace("{loader_version}", loader_version.as_str())
    }

    fn create_profile(&self, mc_version: String, loader_version: String) -> Result<Main, String> {
        let url = JSON_PROFILE
            .replace(":game_version", mc_version.as_str())
            .replace(":loader_version", loader_version.as_str());

        let raw_profile = path::read_file_from_url_to_string(url.as_str())?;

        match version::parse_version_manifest(&raw_profile) {
            Ok(main) => Ok(main),
            Err(err) => Err(err.to_string()),
        }
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        Box::new(FabricInstaller::new())
    }
}

fn get_versions(string: &str) -> Result<Vec<VersionInfo>, String> {
    match serde_json::from_str(string) {
        Ok(version_infos) => Ok(version_infos),
        Err(err) => Err(err.to_string()),
    }
}

fn get_loader_versions(string: &str) -> Result<Vec<LoaderVersionInfo>, String> {
    match serde_json::from_str(string) {
        Ok(version_infos) => Ok(version_infos),
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Deserialize)]
struct VersionInfo {
    pub version: String,
    pub stable: bool,
}

#[derive(Deserialize)]
struct LoaderVersionInfo {
    pub loader: LoaderInfo,
}

#[derive(Deserialize)]
struct LoaderInfo {
    pub version: String,
}
