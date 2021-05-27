use crate::minecraft_launcher::manifest::version::Main;
use std::collections::HashMap;

// This shouldn't be implemented in the first version of the launcher.
mod cursed_legacy;
mod fabric;
mod legacy_fabric;
mod liteloader;
mod rift;

pub trait ModLoaderInstaller {
    fn get_name(&self) -> String;

    fn get_compatible_versions(&self) -> Result<Vec<String>, String>;

    fn save_compatible_versions(&self) -> bool {
        false
    }

    fn get_loader_versions(&self, mc_version: String) -> Result<HashMap<String, String>, String>;

    fn save_compatible_loader_versions(&self) -> bool {
        false
    }

    fn get_profile_name_for_mc_version(&self, mc_version: String) -> String;

    fn get_profile_name_for_loader_version(
        &self,
        mc_version: String,
        loader_version: String,
    ) -> String;

    fn create_profile(&self, mc_version: String, loader_version: String) -> Main;

    fn is_vanilla(&self) -> bool {
        false
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller>;
}

#[derive(Clone)]
pub struct VanillaLoader {}

impl ModLoaderInstaller for VanillaLoader {
    fn get_name(&self) -> String {
        "Vanilla".to_string()
    }

    fn get_compatible_versions(&self) -> Result<Vec<String>, String> {
        todo!()
    }

    fn get_loader_versions(&self, mc_version: String) -> Result<HashMap<String, String>, String> {
        todo!()
    }

    fn get_profile_name_for_mc_version(&self, mc_version: String) -> String {
        mc_version
    }

    fn get_profile_name_for_loader_version(
        &self,
        mc_version: String,
        loader_version: String,
    ) -> String {
        self.get_profile_name_for_mc_version(mc_version)
    }

    fn create_profile(&self, mc_version: String, loader_version: String) -> Main {
        todo!()
    }

    fn is_vanilla(&self) -> bool {
        true
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        Box::new(VanillaLoader {})
    }
}

pub struct ModLoaderHandler {
    pub mod_loaders: Vec<Box<dyn ModLoaderInstaller>>,
    pub vanilla: VanillaLoader,
}

impl ModLoaderHandler {
    pub fn new() -> ModLoaderHandler {
        let mut mod_loaders: Vec<Box<dyn ModLoaderInstaller>> = vec![];

        mod_loaders.push(Box::new(liteloader::LiteLoaderInstaller {}));
        mod_loaders.push(Box::new(rift::RiftInstaller {}));
        mod_loaders.push(Box::new(cursed_legacy::CursedLegacyInstaller {}));

        ModLoaderHandler {
            mod_loaders,
            vanilla: VanillaLoader {},
        }
    }

    pub fn get_loaders_for_version(&self, version: String) -> Result<Vec<Box<dyn ModLoaderInstaller>>, String> {
        let mut loaders: Vec<Box<dyn ModLoaderInstaller>> = vec![];

        loaders.push(Box::new(self.vanilla.clone()));

        for mod_loader in self.mod_loaders.iter() {
            if mod_loader.get_compatible_versions()?.contains(&version) {
                loaders.push(mod_loader.clone_instance());
            }
        }

        Ok(loaders)
    }
}
