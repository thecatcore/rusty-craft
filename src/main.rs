use reqwest::blocking::get as get_url;
use std::{
    env::consts,
    fs,
    fs::{
        File,
        ReadDir
    },
    path::{
        PathBuf
    },
    io,
    io::{
        Read,
        Write,
        Error
    },
};
use directories::BaseDirs;

fn main() {
    let base_dir = BaseDirs::new().expect("Can't get base directories!");
    minecraft_folder(&base_dir);
}

fn minecraft_folder(base_dir: &BaseDirs) {
    let minecraft_folder: &PathBuf = &get_minecraft_directory(base_dir);

    let m_dir = match fs::read_dir(minecraft_folder) {
        Ok(read_dir) => {
            println!(".minecraft folder exist.");
            read_dir
        }
        Err(_) => {
            println!(".minecraft folder doesn't exist, creating it...");
            match fs::create_dir(minecraft_folder) {
                Ok(_) => {
                    println!(".minecraft folder was created succefully!");
                    fs::read_dir(minecraft_folder).expect("How")
                }
                Err(_) => {
                    panic!("Failed to create .minecraft folder!");
                }
            }
        }
    };

    let m_entries_buff = match m_dir
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
    {
        Ok(vec) => vec,
        Err(_) => {
            panic!("Unable to read entries inside of .minecraft folder")
        }
    };

    let mut m_entries_name: Vec<&str> = Vec::new();

    for i in &m_entries_buff {
        m_entries_name.push(
            i.file_name()
                .expect("No name?")
                .to_str()
                .expect("Can't turn path into &str?"),
        );
    }

    if !m_entries_name.contains(&"versions") {
        match fs::create_dir(minecraft_folder.join("versions")) {
            Ok(_) => {
                println!("Successfully created versions folder.")
            }
            Err(_) => {
                panic!("Failed to create versions folder!")
            }
        };
        m_entries_name.push("versions");
    }

    let version_folder = minecraft_folder.join("versions");

    upgrade_manifest(&version_folder);

    get_local_versions(&version_folder);
}

fn upgrade_manifest(version_folder: &PathBuf) {
    let manifest_path = version_folder.join("version_manifest_v2.json");

    let manifest_body = match get_body_from_url_else_from_file(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
        &manifest_path,
    ) {
        None => {
            panic!("Unable to get body of versions manifest!")
        }
        Some(string) => string,
    };

    let mut manifest_file =
        File::create(&manifest_path)
            .expect("Error with manifest file");
    manifest_file.write(manifest_body.as_bytes());

    println!("Successfully updated version manifest.")
}

fn get_body_from_url_else_from_file(url: &str, path: &PathBuf) -> Option<String> {
    match get_url(url) {
        Ok(mut response) => {
            let mut manifest_body = String::new();
            match response.read_to_string(&mut manifest_body) {
                Ok(_) => Option::Some(manifest_body),
                Err(_) => Option::None,
            }
        }
        Err(_) => {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(_) => {
                    return Option::None;
                }
            };

            let mut body = String::new();
            match file.read_to_string(&mut body) {
                Ok(_) => Option::Some(body),
                Err(_) => Option::None,
            }
        }
    }
}

fn get_minecraft_directory_name() -> &'static str {
    match consts::OS {
        "macos" => "minecraft",
        &_ => ".minecraft",
    }
}

fn get_minecraft_directory(base_dir: &BaseDirs) -> PathBuf {
    let dir = match consts::OS {
        "windows" => base_dir.data_dir(),
        "macos" => base_dir.data_dir(),
        &_ => base_dir.home_dir(),
    };

    let min_dir = dir.join(get_minecraft_directory_name());

    min_dir
}

fn get_local_versions(version_folder: &PathBuf) {
    let version_read = match fs::read_dir(version_folder) {
        Ok(read_dir) => {read_dir}
        Err(_) => {panic!("Version folder is supposed to exist but cannot be accessed???")}
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
    let mut m_entries_path_buff: Vec<&PathBuf> = Vec::new();

    for i in &m_entries_buff {
        let nm = i.file_name()
            .expect("No name?")
            .to_str()
            .expect("Can't turn path into &str?");

        if i.is_dir() {
            m_entries_name.push(nm);
            m_entries_path_buff.push(i);
        }
    }

    for i in m_entries_name {
        println!("{}", i)
    }
}
