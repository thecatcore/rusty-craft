use crate::minecraft_launcher::manifest::version::{Arguments, Either, Library, Main, VersionType};
use crate::minecraft_launcher::modding::ModLoaderInstaller;
use chrono::Utc;
use std::collections::HashMap;

const MC_VERSIONS: &str = "1.13.2";

const LIBS: [(&str, &str); 6] = [
    (
        "com.github.unascribed:Rift:FINAL",
        "https://www.jitpack.io/",
    ),
    (
        "org.dimdev:mixin:0.7.11-evil",
        "https://repo.unascribed.com/",
    ),
    (
        "org.ow2.asm:asm:6.2",
        "https://repo.spongepowered.org/maven/",
    ),
    (
        "org.ow2.asm:asm-commons:6.2",
        "https://repo.spongepowered.org/maven/",
    ),
    (
        "org.ow2.asm:asm-tree:6.2",
        "https://repo.spongepowered.org/maven/",
    ),
    ("net.minecraft:launchwrapper:1.12", ""),
];

#[derive(Clone)]
pub struct RiftInstaller {}

impl RiftInstaller {
    pub fn new() -> RiftInstaller {
        RiftInstaller {}
    }
}

impl ModLoaderInstaller for RiftInstaller {
    fn get_name(&self) -> String {
        "Rift".to_string()
    }

    fn get_compatible_versions(&self) -> Result<Vec<String>, String> {
        Ok(vec![String::from(MC_VERSIONS)])
    }

    fn get_loader_versions(&self, _mc_version: String) -> Result<HashMap<String, String>, String> {
        let mut map = HashMap::new();
        map.insert("FINAL".to_string(), "Unknown".to_string());

        Ok(map)
    }

    fn get_profile_name_for_mc_version(&self, _mc_version: String) -> String {
        String::from("1.13.2-rift-FINAL")
    }

    fn get_profile_name_for_loader_version(
        &self,
        mc_version: String,
        _loader_version: String,
    ) -> String {
        self.get_profile_name_for_mc_version(mc_version)
    }

    fn create_profile(&self, mc_version: String, _loader_version: String) -> Main {
        let mut libs: Vec<Library> = Vec::new();
        for lib in LIBS.iter() {
            libs.push(Library {
                downloads: None,
                name: String::from(lib.0),
                extract: None,
                natives: None,
                rules: None,
                url: if lib.1.is_empty() {
                    None
                } else {
                    Some(String::from(lib.1))
                },
            });
        }

        Main {
            arguments: Some(Arguments {
                game: vec![
                    Either::Left(String::from("--tweakClass")),
                    Either::Left(String::from(
                        "org.dimdev.riftloader.launch.RiftLoaderClientTweaker",
                    )),
                ],
                jvm: None,
            }),
            asset_index: None,
            assets: None,
            compliance_level: None,
            downloads: None,
            id: self.get_profile_name_for_mc_version(mc_version.clone()),
            java_version: None,
            libraries: libs,
            logging: None,
            main_class: String::from("net.minecraft.launchwrapper.Launch"),
            minimum_launcher_version: None,
            release_time: Utc::now(),
            time: Utc::now(),
            _type: VersionType::Release,
            minecraft_arguments: None,
            inherits_from: Some(mc_version),
        }
    }

    fn clone_instance(&self) -> Box<dyn ModLoaderInstaller> {
        Box::new(RiftInstaller::new())
    }
}
