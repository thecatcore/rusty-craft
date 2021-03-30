use crate::minecraft_launcher::{
    manifest::assets,
    manifest::assets::Main,
    manifest::version,
    manifest::version::{
        Downloads,
        AssetIndex
    },
    path
};

use std::path::PathBuf;
use std::fs::{Metadata, File};
use std::io::{Error, Read};

pub fn install_client_jar(version_manifest: &version::Main) {
    let version_manifest = version_manifest.clone();

    match version_manifest.downloads {
        None => {
            println!("No client jar to download in version manifest!");
        }
        Some(d) => {
            let client_entry = d.client;
            match path::get_version_folder(&version_manifest.id) {
                None => {
                    println!("Unable to access or create version folder");
                }
                Some(v_path) => {
                    let jar_path = v_path.join(version_manifest.id + &String::from(".jar"));

                    if jar_path.exists() {
                        match jar_path.metadata() {
                            Ok(metadata) => {
                                if metadata.len() != client_entry.size {
                                    match path::download_file_to(&client_entry.url, &jar_path) {
                                        Ok(msg) => {
                                            println!("{}", msg);
                                        }
                                        Err(err) => {
                                            println!("{}", err);
                                        }
                                    };
                                }
                            }
                            Err(err) => {
                                println!("Error while trying to get client jar metadata: {}", err)
                            }
                        }
                    } else {
                        match path::download_file_to(&client_entry.url, &jar_path) {
                            Ok(msg) => {
                                println!("{}", msg);
                            }
                            Err(err) => {
                                println!("{}", err);
                            }
                        };
                    }
                }
            }
        }
    };
}

const LIBRARY_URL: &str = "https://libraries.minecraft.net";

pub fn install_libraries(version_manifest: &version::Main) {
    let version_manifest = version_manifest.clone();


}

pub fn install_assets_index(version_manifest: &version::Main) {
    let version_manifest = version_manifest.clone();

    match version_manifest.asset_index {
        None => {}
        Some(a_index) => {
            match path::get_assets_folder(&String::from("indexes")) {
                None => {}
                Some(index_folder) => {
                    let index_file = index_folder.join(format!("{}.json", &a_index.id));

                    if index_file.exists() {
                        match index_file.metadata() {
                            Ok(meta) => {
                                if meta.len() != a_index.size {
                                    match path::download_file_to(&a_index.url, &index_file) {
                                        Ok(msg) => {
                                            update_assets(a_index.id)
                                        }
                                        Err(err_msg) => {}
                                    }
                                }
                            }
                            Err(err) => {}
                        }
                    } else {
                        match path::download_file_to(&a_index.url, &index_file) {
                            Ok(msg) => {
                                update_assets(a_index.id)
                            }
                            Err(err_msg) => {}
                        }
                    }
                }
            }
        }
    }
}

fn update_assets(index: String) {
    match path::get_assets_folder(&String::from("indexes")) {
        None => {}
        Some(index_folder) => {
            let index_file = index_folder.join(format!("{}.json", index));

            if index_file.exists() {
                match File::open(index_file) {
                    Ok(mut index_file) => {
                        let mut body = String::new();
                        match index_file.read_to_string(&mut body) {
                            Ok(_) => {}
                            Err(_) => {panic!()}
                        };

                        match assets::parse(&body) {
                            Ok(main) => {
                                match path::get_assets_folder(&String::from("objects")) {
                                    None => {}
                                    Some(object_path) => {
                                        for entry in main.objects {
                                            let asset_path = entry.1.get_download_path(&object_path);

                                            if asset_path.exists() {
                                                match asset_path.metadata() {
                                                    Ok(meta) => {
                                                        if meta.len() != entry.1.size {
                                                            match path::download_file_to(&entry.1.get_download_url(), &asset_path) {
                                                                Ok(msg) => {}
                                                                Err(err) => {}
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {}
                                                }
                                            } else {
                                                match path::download_file_to(&entry.1.get_download_url(), &asset_path) {
                                                    Ok(msg) => {}
                                                    Err(err) => {}
                                                }
                                            }
                                        }
                                    }
                                };
                            }
                            Err(err) => {}
                        }
                    }
                    Err(err) => {}
                }
            }
        }
    }
}