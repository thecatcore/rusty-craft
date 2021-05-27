use crate::minecraft_launcher::modding::ModLoaderInstaller;
use std::collections::HashMap;
use crate::minecraft_launcher::manifest::version::Main;

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
        todo!()
    }

    fn save_compatible_versions(&self) -> bool {
        true
    }

    fn get_loader_versions(&self, mc_version: String) -> Result<HashMap<String, String>, String> {
        todo!()
    }

    fn save_compatible_loader_versions(&self) -> bool {
        true
    }

    fn get_profile_name_for_mc_version(&self, mc_version: String) -> String {
        PROFILE_NAMING.replace("{mc_version}", mc_version.as_str())
    }

    fn get_profile_name_for_loader_version(&self, mc_version: String, loader_version: String) -> String {
        self.get_profile_name_for_mc_version(mc_version).replace("{loader_version}", loader_version.as_str())
    }

    fn create_profile(&self, mc_version: String, loader_version: String) -> Main {
        todo!()
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        todo!()
    }
}

struct VersionInfo {
    pub version: String,
    pub stable: bool
}