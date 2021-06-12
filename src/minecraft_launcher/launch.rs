use crate::minecraft_launcher::app::download_tab::Message;
use crate::minecraft_launcher::arguments;
use crate::minecraft_launcher::manifest::version::{
    LibraryDownloadArtifact, Main, Rule, RuleAction,
};
use crate::minecraft_launcher::path;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;

pub fn main(java_path: PathBuf, args: Vec<String>) -> Child {
    Command::new(java_path)
        .current_dir(path::get_minecraft_directory())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Unable to launch Minecraft!")
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
                    None => {
                        vec![]
                    }
                    Some(rules) => rules,
                };
                i += 1;

                match arguments::match_rules(rules, None) {
                    RuleAction::Allow => {
                        let mut classiers: HashMap<String, LibraryDownloadArtifact> =
                            HashMap::new();

                        match library.downloads {
                            None => {}
                            Some(lib_down) => match lib_down.classifiers {
                                None => {}
                                Some(classifiers) => {
                                    classiers = classifiers;
                                }
                            },
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
                            Some(map) => match map.get(arguments::get_os().to_str().as_str()) {
                                None => {}
                                Some(native_name) => match classiers.get(native_name.as_str()) {
                                    None => {}
                                    Some(lib_download_art) => {
                                        let lib_path = lib_download_art.clone().path;
                                        tx.send(Message::NewSubStep(
                                            lib_path.clone().to_string(),
                                            i,
                                            manifest.libraries.len() as u64,
                                        ))
                                        .expect("Can't send message to renderer thread");
                                        match path::get_library_path(&lib_path) {
                                            None => {}
                                            Some(lib_path) => {
                                                if lib_path.exists() {
                                                    tx = extract_natives(
                                                        bin_folder.clone(),
                                                        lib_path,
                                                        exclude,
                                                        tx,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                },
                            },
                        }
                    }
                    RuleAction::Disallow => {}
                }
            }
            tx.send(Message::NewStep(8))
                .expect("Can't send message to renderer thread");
        }
    };
}

fn extract_natives(
    bin_folder: PathBuf,
    lib_path: PathBuf,
    exclude: Option<Vec<String>>,
    tx: Sender<Message>,
) -> Sender<Message> {
    let file = fs::File::open(lib_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut excludes = Vec::new();
    match exclude {
        None => {}
        Some(excl) => {
            excludes = excl;
        }
    }

    let length = archive.len();

    for i in 0..length {
        let mut file = archive.by_index(i).unwrap();

        tx.send(Message::NewSubSubStep(
            file.name().to_string(),
            (i + 1) as u64,
            length as u64,
        ))
        .expect("Can't send message to renderer thread");

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if !excludes.contains(&String::from(file.name()))
            && !excludes.contains(&(String::from(file.name()) + "/"))
        {
            let file_path = outpath.as_os_str();
            if file.is_dir() {
                match path::get_or_create_dir(
                    &bin_folder,
                    String::from(file_path.to_str().unwrap()),
                ) {
                    None => {}
                    Some(_) => {}
                }
            } else if file.is_file() {
                if let Ok(mut outfile) = File::create(bin_folder.join(file_path)) {
                    io::copy(&mut file, &mut outfile).unwrap();
                }
            }
        }
    }

    tx
}
