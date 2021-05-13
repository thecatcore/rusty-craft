use crate::minecraft_launcher::manifest::version::Main;

// This shouldn't be implemented in the first version of the launcher.
mod cursed_legacy;
mod fabric;
mod legacy_fabric;
mod liteloader;
mod rift;

pub trait ModLoaderInstaller {
    fn get_compatible_versions(&self) -> Vec<String>;

    fn get_loader_versions(&self, mc_version: String) -> Vec<String>;

    fn get_profile_name_for_mc_version(&self, mc_version: String) -> String;

    fn get_profile_name_for_loader_version(
        &self,
        mc_version: String,
        loader_version: String,
    ) -> String;

    fn create_profile(&self, mc_version: String, loader_version: String) -> Main;
}
