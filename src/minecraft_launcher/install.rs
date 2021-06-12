use crate::minecraft_launcher::{
    arguments, launch,
    // manifest::assets,
    manifest::version,
    manifest::version::{LibraryDownloadArtifact, RuleAction},
    path,
};

use crate::minecraft_launcher::app::download_tab::Message;
use crate::minecraft_launcher::manifest;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use crate::minecraft_launcher::modding::ModLoaderInstaller;
use crate::minecraft_launcher::manifest::version::Main;
use serde_json::Error;

pub(crate) mod java;
pub(crate) mod assets;

pub fn install_version(
    id: String,
    versions: Vec<manifest::main::Version>,
    tx: Sender<Message>,
    modded_version: Option<Main>
) -> Option<()> {
    tx.send(Message::NewSubStep(
        String::from("Checking Version folder"),
        1,
        3,
    ))
    .unwrap_or(());

    match modded_version {
        // Vanilla version
        None => if let Some(version_folder) = path::get_version_folder(&id) {
            let manifest_file_path = version_folder.join(id.clone() + ".json");
            if manifest_file_path.exists() {
                read_version_manifest_and_install(manifest_file_path, tx)
            } else {
                for version in versions {
                    if version.id == id {
                        return download_and_install_vanilla(version, manifest_file_path, tx);
                    }
                }
                None
            }
        } else {
            None
        }

        // Modded version
        Some(mut modded_version) => match modded_version.clone().inherits_from {
            None => {None}
            Some(inherit_from) => if let Some(version_folder) = path::get_version_folder(&inherit_from) {
                let manifest_file_path = version_folder.join(inherit_from.clone() + ".json");
                if manifest_file_path.exists() {
                    // read_version_manifest_and_install(manifest_file_path, tx)
                    match read_version_manifest(manifest_file_path, tx.clone()) {
                        None => {None}
                        Some(vanilla_version) => {
                            let version = Main::inherit(modded_version, &vanilla_version);
                            write_version_manifest(&version);
                            install_version_from_manifest(&version, tx)
                        }
                    }
                } else {
                    // for version in versions {
                    //     if version.id == id {
                    //         return download_and_install_vanilla(version, manifest_file_path, tx);
                    //     }
                    // }
                    None
                }
            } else {
                None
            }
        }
    }
}

fn write_version_manifest(version_manifest: &Main) {
    match version::serialize_version_manifest(version_manifest) {
        Ok(version_manifest_str) => {
            if let Some(version_folder) = path::get_version_folder(&version_manifest.id) {
                let manifest_file_path = version_folder.join(version_manifest.id.clone() + ".json");

                let mut file = if manifest_file_path.exists() {
                    File::open(manifest_file_path)
                } else {
                    File::create(manifest_file_path)
                };

                match file {
                    Ok(mut file) => {
                        match file.write(version_manifest_str.as_bytes()) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    }
}

fn download_and_install_vanilla(
    version: manifest::main::Version,
    file_path: PathBuf,
    tx: Sender<Message>,
) -> Option<()> {
    tx.send(Message::NewSubStep(
        String::from("Downloading Version manifest"),
        2,
        3,
    ))
        .unwrap_or(());

    match path::download_file_to(&version.url, &file_path) {
        Ok(_) => read_version_manifest_and_install(file_path, tx),
        Err(_) => None,
    }
}

fn read_version_manifest(manifest_path: PathBuf, tx: Sender<Message>) -> Option<Main> {
    if let Ok(mut file) = File::open(manifest_path) {
        let mut body = String::new();

        if file.read_to_string(&mut body).is_ok() {
            if let Ok(version) = manifest::version::parse_version_manifest(&body) {
                return Some(version);
            }
        }
    }

    None
}

fn read_version_manifest_and_install(manifest_path: PathBuf, tx: Sender<Message>) -> Option<()> {
    tx.send(Message::NewSubStep(
        String::from("Reading Version manifest"),
        3,
        3,
    ))
    .unwrap_or(());

    match read_version_manifest(manifest_path, tx.clone()) {
        None => None,
        Some(version) => install_version_from_manifest(&version, tx)
    }
}

fn install_version_from_manifest(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<()> {
    if let Some(tx) = java::check_java_version(version_manifest, tx) {
        if let Some(tx) = install_client_jar(version_manifest, tx) {
            if let Some(tx) = install_libraries(version_manifest, tx) {
                if let Some(tx) = assets::install_assets_index(version_manifest, tx) {
                    if let Some(tx) = check_log_file(version_manifest, tx) {
                        launch::pre_launch(version_manifest.clone(), tx.clone());
                        tx.send(Message::Done(version_manifest.clone()))
                            .unwrap_or(());
                        return Some(());
                    }
                }
            }
        }
    }

    None
}

fn install_client_jar(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();
    tx.send(Message::NewStep(3)).unwrap_or(());

    match version_manifest.downloads {
        None => {
            tx.send(Message::Error(
                "No client jar to download in version manifest!".to_string(),
            ))
            .unwrap_or(());
            None
        }
        Some(d) => {
            let client_entry = d.client;
            match path::get_version_folder(&version_manifest.id) {
                None => {
                    tx.send(Message::Error(
                        "Unable to access or create version folder".to_string(),
                    ))
                    .unwrap_or(());
                    None
                }
                Some(v_path) => {
                    let jar_path = v_path.join(version_manifest.id + &String::from(".jar"));

                    if jar_path.exists() {
                        match jar_path.metadata() {
                            Ok(metadata) => {
                                if metadata.len() != client_entry.size {
                                    match path::download_file_to(&client_entry.url, &jar_path) {
                                        Ok(_) => Some(tx),
                                        Err(err) => {
                                            tx.send(Message::Error(err)).unwrap_or(());
                                            None
                                        }
                                    }
                                } else {
                                    Some(tx)
                                }
                            }
                            Err(err) => {
                                tx.send(Message::Error(format!(
                                    "Error while trying to get client jar metadata: {}",
                                    err
                                )))
                                .unwrap_or(());
                                None
                            }
                        }
                    } else {
                        match path::download_file_to(&client_entry.url, &jar_path) {
                            Ok(_) => Some(tx),
                            Err(err) => {
                                tx.send(Message::Error(err)).unwrap_or(());
                                None
                            }
                        }
                    }
                }
            }
        }
    }
}

fn install_libraries(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();

    tx.send(Message::NewStep(4)).unwrap_or(());
    let mut result = Some(());

    let library_count = version_manifest.libraries.len();

    let mut index = 0;

    for library in version_manifest.libraries {
        index += 1;
        let lib_name: Vec<&str> = library.name.split(':').collect();
        let group = *lib_name.get(0).expect("Library doesn't have a group???");
        let name = *lib_name.get(1).expect("Library doesn't have a name???");
        let version = *lib_name.get(2).expect("Library doesn't have a version???");

        tx.send(Message::NewSubStep(
            format!("{}-{}", &name, &version),
            index,
            library_count as u64,
        ))
        .unwrap_or(());

        let allowed = match library.rules {
            None => RuleAction::Allow,
            Some(rules) => arguments::match_rules(rules, None),
        };

        match allowed {
            RuleAction::Allow => {
                let mut downloaded = false;
                let mut classifiers: HashMap<String, LibraryDownloadArtifact> = HashMap::new();

                match library.downloads {
                    None => {}
                    Some(downloads) => {
                        match downloads.artifact {
                            None => {}
                            Some(artifact) => {
                                downloaded = true;
                                match path::get_library_path(&artifact.path) {
                                    None => {
                                        result = None;
                                        break;
                                    }
                                    Some(lib_path) => {
                                        if lib_path.exists() && lib_path.is_file() {
                                            match lib_path.metadata() {
                                                Ok(meta) => {
                                                    if meta.len() != artifact.size {
                                                        match path::download_file_to(
                                                            &artifact.url,
                                                            &lib_path,
                                                        ) {
                                                            Ok(_) => {}
                                                            Err(err) => {
                                                                tx.send(Message::Error(err))
                                                                    .unwrap_or(());
                                                                result = None;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(meta_err) => {
                                                    tx.send(Message::Error(format!(
                                                        "{}",
                                                        meta_err
                                                    )))
                                                    .unwrap_or(());
                                                    result = None;
                                                    break;
                                                }
                                            }
                                        } else {
                                            match path::download_file_to(&artifact.url, &lib_path) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    tx.send(Message::Error(err)).unwrap_or(());
                                                    result = None;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        match downloads.classifiers {
                            None => {}
                            Some(class) => {
                                downloaded = true;
                                for classier in class {
                                    classifiers.insert(classier.0, classier.1);
                                }
                            }
                        }
                    }
                }

                match library.natives {
                    None => {}
                    Some(nat) => match nat.get(arguments::get_os().to_str().as_str()) {
                        None => {}
                        Some(nat_name) => match classifiers.get(nat_name) {
                            None => {
                                result = None;
                                break;
                            }
                            Some(class) => match path::get_library_path(&class.path) {
                                None => {
                                    result = None;
                                    break;
                                }
                                Some(lib_path) => {
                                    if lib_path.exists() && lib_path.is_file() {
                                        match lib_path.metadata() {
                                            Ok(meta) => {
                                                if meta.len() != class.size {
                                                    match path::download_file_to(
                                                        &class.url, &lib_path,
                                                    ) {
                                                        Ok(_) => {}
                                                        Err(err) => {
                                                            tx.send(Message::Error(err))
                                                                .unwrap_or(());
                                                            result = None;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(meta_err) => {
                                                tx.send(Message::Error(format!("{}", meta_err)))
                                                    .unwrap_or(());
                                                result = None;
                                                break;
                                            }
                                        }
                                    } else {
                                        match path::download_file_to(&class.url, &lib_path) {
                                            Ok(_) => {}
                                            Err(err) => {
                                                tx.send(Message::Error(err)).unwrap_or(());
                                                result = None;
                                                break;
                                            }
                                        }
                                    }
                                }
                            },
                        },
                    },
                }

                match library.url {
                    None => {}
                    Some(url) => {
                        downloaded = true;
                        let url_path = group.replace(".", "/")
                            + "/"
                            + name
                            + "/"
                            + version
                            + "/"
                            + name
                            + "-"
                            + version
                            + ".jar";
                        match path::get_library_path(&url_path) {
                            None => {
                                result = None;
                                break;
                            }
                            Some(lib_path) => {
                                if !lib_path.exists() {
                                    match path::download_file_to(
                                        &(url + "/" + &*url_path),
                                        &lib_path,
                                    ) {
                                        Ok(_) => {}
                                        Err(ohno) => {
                                            tx.send(Message::Error(ohno)).unwrap_or(());
                                            result = None;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !downloaded {
                    let url_path =
                        group.replace(".", "/") + "/" + name + "/" + version + "/" + name + "-" + version + ".jar";
                    match path::get_library_path(&url_path) {
                        None => {
                            result = None;
                            break;
                        }
                        Some(lib_path) => {
                            if !lib_path.exists() {
                                match path::download_file_to(
                                    &(String::from("https://libraries.minecraft.net/")
                                        + &*url_path),
                                    &lib_path,
                                ) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        tx.send(Message::Error(err)).unwrap_or(());
                                        result = None;
                                        break;
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

    match result {
        None => None,
        Some(_) => Some(tx),
    }
}

fn check_log_file(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();
    tx.send(Message::NewStep(6)).unwrap_or(());
    match version_manifest.logging {
        None => {
            // println!("No logging, that's fine");
            Some(tx)
        }
        Some(logging) => match logging.client {
            None => {
                // println!("No logging (2), that's fine");
                Some(tx)
            }
            Some(client_log) => {
                let file_info = client_log.file;
                match path::get_assets_folder(&String::from("log_configs")) {
                    None => {
                        tx.send(Message::Error(
                            "Unable to get log_configs folder".to_string(),
                        ))
                        .unwrap_or(());
                        None
                    }
                    Some(log_folder) => {
                        let log_path = log_folder.join(file_info.id);
                        if log_path.exists() {
                            match log_path.metadata() {
                                Ok(meta) => {
                                    if meta.len() != file_info.size {
                                        match path::download_file_to(&file_info.url, &log_path) {
                                            Ok(_) => Some(tx),
                                            Err(err) => {
                                                tx.send(Message::Error(format!(
                                                    "Unable to download logger file: {}",
                                                    err
                                                )))
                                                .unwrap_or(());
                                                None
                                            }
                                        }
                                    } else {
                                        Some(tx)
                                    }
                                }
                                Err(_) => match path::download_file_to(&file_info.url, &log_path) {
                                    Ok(_) => Some(tx),
                                    Err(err) => {
                                        tx.send(Message::Error(format!(
                                            "Unable to download logger file: {}",
                                            err
                                        )))
                                        .unwrap_or(());
                                        None
                                    }
                                },
                            }
                        } else {
                            match path::download_file_to(&file_info.url, &log_path) {
                                Ok(_) => Some(tx),
                                Err(err) => {
                                    tx.send(Message::Error(format!(
                                        "Unable to download logger file: {}",
                                        err
                                    )))
                                    .unwrap_or(());
                                    None
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}
