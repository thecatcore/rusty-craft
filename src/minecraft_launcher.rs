use std::{
    fs,
    fs::File,
    io,
    io::{Read, Write},
    path::PathBuf,
};

mod app;
mod arguments;
mod install;
mod launch;
mod manifest;
mod modding;
mod options;
mod path;
mod rendering;
mod utils;

use manifest::version;
use std::path::Path;

pub fn main() {
    minecraft_folder();
}

fn minecraft_folder() {
    let minecraft_folder: PathBuf = path::get_minecraft_directory();

    match &minecraft_folder.exists() {
        true => {
            println!(".minecraft folder exist.");
        }
        false => {
            println!(".minecraft folder doesn't exist, creating it...");
            match fs::create_dir(&minecraft_folder) {
                Ok(_) => {
                    println!(".minecraft folder was created successfully!");
                }
                Err(_) => {
                    panic!("Failed to create .minecraft folder!");
                }
            }
        }
    };

    let version_folder =
        match path::get_or_create_dir(&minecraft_folder, "versions".parse().unwrap()) {
            Some(p) => p,
            None => panic!("Unable to access or create versions folder"),
        };

    let mut manifest = upgrade_manifest(&version_folder);
    manifest
        .versions
        .sort_by(|a, b| a.release_time.cmp(&b.release_time));
    manifest.versions.reverse();
    let installed = get_local_versions(&version_folder);
    let mut installed_id: Vec<String> = Vec::new();
    println!("Installed versions:");
    for version in &installed {
        println!("Version {} of type {}", version.id, version._type);
        installed_id.push(version.id.clone());
    }
    let mut all_versions: Vec<manifest::main::MinVersion> = Vec::new();

    for version in &installed {
        if !version.is_modded() {
            all_versions.push(version.to_min_version());
        }
    }

    for version in &manifest.versions {
        if !installed_id.contains(&version.id) {
            all_versions.push(version.to_min_version());
        }
    }

    all_versions.sort_by(|a, b| a.release_time.cmp(&b.release_time));
    all_versions.reverse();

    let app = app::App::new(all_versions, manifest.versions);

    match rendering::main::main(app) {
        Ok(_) => {
            println!("It went ok!");
        }
        Err(err) => {
            println!("It went wrong! {}", err);
        }
    };
}

fn upgrade_manifest(version_folder: &Path) -> manifest::main::Main {
    let manifest_path = version_folder.join("version_manifest_v2.json");

    let manifest_body = match utils::get_body_from_url_else_from_file(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
        &manifest_path,
    ) {
        None => {
            panic!("Unable to get body of versions manifest!")
        }
        Some(string) => string,
    };

    let mut manifest_file = File::create(&manifest_path).expect("Error with manifest file");
    match manifest_file.write(&manifest_body.as_bytes()) {
        Ok(_) => {
            println!("Successfully updated version manifest.")
        }
        Err(_) => {
            println!("Failed to update version manifest.")
        }
    };

    match manifest::main::parse_manifest(&manifest_body) {
        Ok(mani) => mani,
        Err(err) => {
            panic!("Manifest wrongly formatted! {}", err)
        }
    }
}

fn get_local_versions(version_folder: &Path) -> Vec<version::Main> {
    let version_read = match fs::read_dir(version_folder) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            panic!("Version folder is supposed to exist but cannot be accessed???")
        }
    };

    let m_entries_buff = match version_read
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
    {
        Ok(vec) => vec,
        Err(_) => {
            panic!("Unable to read entries inside of versions folder")
        }
    };

    let mut m_entries_name: Vec<&str> = Vec::new();

    for i in &m_entries_buff {
        let nm = i
            .file_name()
            .expect("No name?")
            .to_str()
            .expect("Can't turn path into &str?");

        if i.is_dir() {
            m_entries_name.push(nm);
        }
    }

    let mut installed: Vec<version::Main> = Vec::new();

    for i in 0..m_entries_name.len() {
        match get_manifest_from_installed(
            m_entries_name.get(i).expect("Concern"),
            match &path::get_or_create_dir(
                version_folder,
                m_entries_name.get(i).expect("Concern").parse().unwrap(),
            ) {
                None => {
                    panic!("Unable to access or create version folder")
                }
                Some(p) => p,
            },
        ) {
            None => {}
            Some(body) => {
                match version::parse_version_manifest(&body) {
                    Ok(main) => {
                        installed.push(main);
                    }
                    Err(err) => {
                        println!(
                            "Error while parsing version manifest ({}): {}",
                            m_entries_name.get(i).expect("Concern"),
                            err.to_string()
                        );
                    }
                };
            }
        };
    }

    installed.sort_by(|a, b| a.release_time.cmp(&b.release_time));
    installed.reverse();

    installed
}

fn get_manifest_from_installed(version_name: &str, version_folder: &Path) -> Option<String> {
    let version_read = match fs::read_dir(version_folder) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            panic!("Version folder is supposed to exist but cannot be accessed???")
        }
    };

    let m_entries_buff = match version_read
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
    {
        Ok(vec) => vec,
        Err(_) => {
            panic!("Unable to read entries inside of versions folder")
        }
    };

    let mut m_entries_name: Vec<String> = Vec::new();
    let mut m_entries_path_buff: Vec<&PathBuf> = Vec::new();

    for i in &m_entries_buff {
        let nm = i
            .file_name()
            .expect("No name?")
            .to_str()
            .expect("Can't turn path into &str?");

        if i.is_file() {
            m_entries_name.push(nm.to_string());
            m_entries_path_buff.push(i);
        }
    }

    if m_entries_name.contains(&(String::from(version_name) + ".json")) {
        let mut file = match File::open(version_folder.join(String::from(version_name) + ".json")) {
            Ok(file) => file,
            Err(_) => {
                return Option::None;
            }
        };

        let mut body = String::new();
        return match file.read_to_string(&mut body) {
            Ok(_) => return Some(body),
            Err(_) => None,
        };
    }

    None
}
