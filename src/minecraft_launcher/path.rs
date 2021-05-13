use directories::BaseDirs;
use reqwest::blocking::get as get_url;
use std::env::consts;
use std::fs;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::{PathBuf, Path};

pub fn get_or_create_dir(current_folder: &Path, sub: String) -> Option<PathBuf> {
    match current_folder.exists() {
        true => {
            let sub_path = current_folder.join(sub);

            match sub_path.exists() {
                true => Some(sub_path),
                false => match fs::create_dir_all(&sub_path) {
                    Ok(_) => Some(sub_path),
                    Err(e) => {
                        print!(
                            "Unable to create directory {} in folder {}: {}",
                            sub_path.file_name().expect("Ohno").to_str().expect("REE"),
                            current_folder
                                .file_name()
                                .expect("Ohno")
                                .to_str()
                                .expect("REE"),
                            e.to_string()
                        );
                        None
                    }
                },
            }
        }
        false => match fs::create_dir_all(current_folder) {
            Ok(_) => {
                let sub_path = current_folder.join(sub);

                match sub_path.exists() {
                    true => Some(sub_path),
                    false => match fs::create_dir_all(&sub_path) {
                        Ok(_) => Some(sub_path),
                        Err(e) => {
                            print!(
                                "Unable to create directory {} in folder {}: {}",
                                sub_path.file_name().expect("Ohno").to_str().expect("REE"),
                                current_folder
                                    .file_name()
                                    .expect("Ohno")
                                    .to_str()
                                    .expect("REE"),
                                e.to_string()
                            );
                            None
                        }
                    },
                }
            }
            Err(e) => {
                print!(
                    "Unable to create folder {}: {}",
                    current_folder
                        .file_name()
                        .expect("Ohno")
                        .to_str()
                        .expect("REE"),
                    e.to_string()
                );
                None
            }
        },
    }
}

pub fn download_file_to(url: &str, path: &Path) -> Result<String, String> {
    match read_file_from_url_to_type(url, AskedType::U8Vec) {
        Ok(u8_vec) => match u8_vec {
            ReturnType::U8Vec(body) => {
                let mut file = if path.exists() {
                    match File::open(path) {
                        Ok(file) => file,
                        Err(err) => {
                            return Err(format!(
                                "Failed to download (open) {} to {}: {}",
                                url,
                                path.to_str().unwrap(),
                                err
                            ));
                        }
                    }
                } else {
                    match File::create(path) {
                        Ok(file) => file,
                        Err(err) => {
                            return Err(format!(
                                "Failed to download (create) {} to {}: {}",
                                url,
                                path.to_str().unwrap(),
                                err
                            ));
                        }
                    }
                };

                match file.write(&body) {
                    Ok(_) => Ok(format!(
                        "Successfully wrote {} to {}",
                        url,
                        path.file_name().expect("Ohno").to_str().expect("OhnoV2")
                    )),
                    Err(err) => Err(format!(
                        "Failed to download (write) {} to {}: {}",
                        url,
                        path.to_str().unwrap(),
                        err
                    )),
                }
            }
            ReturnType::String(_) => {
                Err("Wrong Return type, expected Vec<u8> found String!".to_string())
            }
        },
        Err(err) => Err(format!(
            "Failed to download {} to {}: {}",
            url,
            path.to_str().unwrap(),
            match err {
                ErrorType::STD(e) => {
                    e.to_string()
                }
                ErrorType::Reqwest(e) => {
                    e.to_string()
                }
            }
        )),
    }
}

pub fn read_file_from_url_to_string(url: &str) -> Result<String, String> {
    match read_file_from_url_to_type(url, AskedType::String) {
        Ok(string) => match string {
            ReturnType::U8Vec(_) => {
                Err("Wrong Return type, expected String found Vec<u8>!".to_string())
            }
            ReturnType::String(string) => Ok(string),
        },
        Err(err) => Err(format!(
            "Failed to download {}: {}",
            url,
            match err {
                ErrorType::STD(e) => {
                    e.to_string()
                }
                ErrorType::Reqwest(e) => {
                    e.to_string()
                }
            }
        )),
    }
}

pub fn read_file_from_url_to_type(url: &str, type_: AskedType) -> Result<ReturnType, ErrorType> {
    match get_url(url) {
        Ok(mut data) => match type_ {
            AskedType::U8Vec => {
                let mut body: Vec<u8> = Vec::new();
                match data.read_to_end(&mut body) {
                    Ok(_) => Ok(ReturnType::U8Vec(body)),
                    Err(err) => Err(ErrorType::STD(err)),
                }
            }
            AskedType::String => {
                let mut body = String::new();
                match data.read_to_string(&mut body) {
                    Ok(_) => Ok(ReturnType::String(body)),
                    Err(err) => Err(ErrorType::STD(err)),
                }
            }
        },
        Err(err) => Err(ErrorType::Reqwest(err)),
    }
}

pub enum ReturnType {
    U8Vec(Vec<u8>),
    String(String),
}

pub enum ErrorType {
    STD(Error),
    Reqwest(reqwest::Error),
}

pub enum AskedType {
    U8Vec,
    String,
}

pub fn get_version_folder(version: &str) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("versions")) {
        None => None,
        Some(vs) => get_or_create_dir(&vs, version.to_string()),
    }
}

pub fn get_assets_folder(sub: &str) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("assets")) {
        None => None,
        Some(vs) => get_or_create_dir(&vs, sub.to_string()),
    }
}

pub fn get_library_path(sub: &str) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("libraries")) {
        None => None,
        Some(vs) => {
            if sub.contains('/') {
                let sub = PathBuf::from(sub);
                match get_or_create_dir(
                    &vs,
                    sub.parent().unwrap().to_str().unwrap().parse().unwrap(),
                ) {
                    None => None,
                    Some(lib_fol) => {
                        Some(lib_fol.join(sub.components().last().unwrap().as_os_str()))
                    }
                }
            } else {
                get_or_create_dir(&vs, sub.to_string())
            }
        }
    }
}

pub fn get_java_folder_path(type_: &str) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("runtime")) {
        None => None,
        Some(runtime) => match get_or_create_dir(&runtime, type_.to_string()) {
            None => None,
            Some(type1) => match get_or_create_dir(&type1, String::from(get_os_java_name())) {
                None => None,
                Some(os) => Some(os),
            },
        },
    }
}

pub fn get_java_folder_path_sub(type_: &str) -> Option<PathBuf> {
    match get_java_folder_path(type_) {
        None => None,
        Some(os) => Some(os.join(type_)),
    }
}

pub fn get_bin_folder(version_name: String) -> Option<PathBuf> {
    get_or_create_dir(
        &get_minecraft_directory(),
        String::from("bin/") + &version_name,
    )
}

fn get_os_java_name() -> &'static str {
    match consts::OS {
        "windows" => match consts::ARCH {
            "x86" => "windows-x86",
            "x86_64" => "windows-x64",
            &_ => "",
        },
        "macos" => "mac-os",
        &_ => match consts::ARCH {
            "x86" => "linux-i386",
            &_ => "linux",
        },
    }
}

pub fn get_minecraft_sub_folder(sub: &str) -> Option<PathBuf> {
    get_or_create_dir(&get_minecraft_directory(), sub.to_string())
}

fn get_minecraft_directory_name() -> &'static str {
    match consts::OS {
        "macos" => "minecraft",
        &_ => ".minecraft",
    }
}

pub fn get_minecraft_directory() -> PathBuf {
    let base_dir: BaseDirs = BaseDirs::new().expect("Can't get base directories!");

    let dir = match consts::OS {
        "windows" => base_dir.data_dir(),
        "macos" => base_dir.data_dir(),
        &_ => base_dir.home_dir(),
    };

    let min_dir = dir.join(get_minecraft_directory_name());

    min_dir
}
