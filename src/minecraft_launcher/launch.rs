use std::path::PathBuf;
use std::process::Command;
use crate::minecraft_launcher::manifest::version::{Main, Rule, RuleAction, LibraryDownload, LibraryDownloadArtifact, LibraryExtract};
use crate::minecraft_launcher::path;
use crate::minecraft_launcher::arguments;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::fs::File;
use std::io::Error;
use std::sync::mpsc::Sender;
use crate::minecraft_launcher::app::download_tab::Message;

pub fn main(java_path: PathBuf) {
    let child = Command::new("");
}

pub fn pre_launch(manifest: Main, mut tx: Sender<Message>) {
    tx.send(Message::NewStep(7))
        .expect("Can't send message to renderer thread");
    match path::get_bin_folder(manifest.id) {
        None => {}
        Some(bin_folder) => {
            let mut i = 0;
            for library in manifest.libraries.clone() {
                let rules: Vec<Rule> = match library.rules {
                    None => {vec![]}
                    Some(rules) => {rules}
                };
                i += 1;

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

                        let mut exclude: Option<Vec<String>> = None;

                        match library.extract {
                            None => {}
                            Some(extr) => {
                                exclude = Some(extr.exclude.clone());
                            }
                        }

                        match library.natives {
                            None => {}
                            Some(map) => {
                                match map.get(arguments::get_os().to_str().as_str()) {
                                    None => {}
                                    Some(native_name) => {
                                        match classiers.get(native_name.as_str()) {
                                            None => {}
                                            Some(lib_download_art) => {
                                                let lib_path = lib_download_art.clone().path;
                                                tx.send(Message::NewSubStep(format!("{}", lib_path.clone()), i, manifest.libraries.len() as u64)).expect("Can't send message to renderer thread");
                                                match path::get_library_path(&lib_path) {
                                                    None => {}
                                                    Some(lib_path) => if lib_path.exists() {
                                                        tx = extract_natives(bin_folder.clone(), lib_path, exclude, tx);
                                                    }
                                                }
                                            }
                                        }
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

fn extract_natives(bin_folder: PathBuf, lib_path: PathBuf, exclude: Option<Vec<String>>, tx: Sender<Message>) -> Sender<Message> {
    let file = fs::File::open(lib_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut excludes = Vec::new();
    match exclude {
        None => {}
        Some(excl) => {excludes = excl;}
    }

    let length = archive.len();

    for i in 0..length {
        let mut file = archive.by_index(i).unwrap();

        tx.send(Message::NewSubSubStep(format!("{}", file.name()), (i + 1) as u64, length as u64)).expect("Can't send message to renderer thread");

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if !excludes.contains(&String::from(file.name())) && !excludes.contains(&(String::from(file.name()) + "/")) {
            let name = String::from(file.name());

            if name.contains("/") {
                let parent = outpath.parent().unwrap().as_os_str().to_str().unwrap();
                match path::get_or_create_dir(&bin_folder, String::from(parent)) {
                    None => {}
                    Some(parent) => {
                        let child = parent.join(outpath.components().last().unwrap().as_os_str());
                        match File::create(child) {
                            Ok(mut outfile) => {
                                io::copy(&mut file, &mut outfile).unwrap();
                            }
                            Err(_) => {}
                        }
                    }
                }
            } else {
                let file_path = outpath.as_os_str();
                match File::create(bin_folder.join(file_path)) {
                    Ok(mut outfile) => {
                        io::copy(&mut file, &mut outfile).unwrap();
                    }
                    Err(_) => {}
                }
            }
        }
    }

    tx
}
