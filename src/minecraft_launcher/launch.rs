use std::path::PathBuf;
use std::process::Command;
use crate::minecraft_launcher::manifest::version::{Main, Rule, RuleAction, LibraryDownload, LibraryDownloadArtifact, LibraryExtract};
use crate::minecraft_launcher::path;
use crate::minecraft_launcher::arguments;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

pub fn main(java_path: PathBuf) {
    let child = Command::new("");
}

fn pre_launch(manifest: Main) {
    match path::get_bin_folder(manifest.id) {
        None => {}
        Some(bin_folder) => {
            for library in manifest.libraries {
                let rules: Vec<Rule> = match library.rules {
                    None => {vec![]}
                    Some(rules) => {rules}
                };

                match arguments::match_rules(rules, None) {
                    RuleAction::Allow => {
                        let mut classiers: HashMap<String, LibraryDownloadArtifact> = HashMap::new();

                        match library.downloads {
                            None => {}
                            Some(lib_down) => {
                                match lib_down.classifiers {
                                    None => {}
                                    Some(classifiers) => {
                                        classiers = classifiers;
                                    }
                                }
                            }
                        }

                        match library.natives {
                            None => {}
                            Some(map) => {
                                match map.get(arguments::get_os().to_str().as_str()) {
                                    None => {}
                                    Some(native_name) => {

                                    }
                                }
                            }
                        }
                    }
                    RuleAction::Disallow => {}
                }
            }
        }
    };
}
