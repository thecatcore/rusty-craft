use crate::minecraft_launcher::{
    arguments,
    manifest::assets,
    manifest::assets::Main,
    manifest::version,
    manifest::version::{
        AssetIndex, Downloads, LibraryDownloadArtifact, LibraryExtract, Rule, RuleAction,
    },
    manifest::java_versions,
    manifest::java,
    path,
};

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{Error, Read};
use std::path::PathBuf;
use crate::minecraft_launcher::manifest::version::JavaVersion;

pub fn install_version(version_manifest: &version::Main) -> Option<()> {

    match install_client_jar(version_manifest) {
        None => {return None;}
        Some(_) => {}
    }

    match install_libraries(version_manifest) {
        None => {return None;}
        Some(_) => {}
    }

    match install_assets_index(version_manifest) {
        None => {return None;}
        Some(_) => {}
    }

    Some(())
}

fn install_client_jar(version_manifest: &version::Main) -> Option<()> {
    let version_manifest = version_manifest.clone();

    match version_manifest.downloads {
        None => {
            println!("No client jar to download in version manifest!");
            None
        }
        Some(d) => {
            let client_entry = d.client;
            match path::get_version_folder(&version_manifest.id) {
                None => {
                    println!("Unable to access or create version folder");
                    None
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
                                            Some(())
                                        }
                                        Err(err) => {
                                            println!("{}", err);
                                            None
                                        }
                                    }
                                } else {
                                    Some(())
                                }
                            }
                            Err(err) => {
                                println!("Error while trying to get client jar metadata: {}", err);
                                None
                            }
                        }
                    } else {
                        match path::download_file_to(&client_entry.url, &jar_path) {
                            Ok(msg) => {
                                println!("{}", msg);
                                Some(())
                            }
                            Err(err) => {
                                println!("{}", err);
                                None
                            }
                        }
                    }
                }
            }
        }
    }
}

fn install_libraries(version_manifest: &version::Main) -> Option<()> {
    let version_manifest = version_manifest.clone();

    let mut result = Some(());

    for library in version_manifest.libraries {
        let lib_name: Vec<&str> = library.name.split(":").collect();
        let group = *lib_name.get(0).expect("Library doesn't have a group???");
        let name = *lib_name.get(1).expect("Library doesn't have a name???");
        let version = *lib_name.get(1).expect("Library doesn't have a version???");

        let allowed = match library.rules {
            None => RuleAction::Allow,
            Some(rules) => arguments::match_rules(rules, None),
        };

        if allowed.to_string() == RuleAction::Allow.to_string() {
            let mut classifiers: HashMap<String, LibraryDownloadArtifact> = HashMap::new();

            match library.downloads {
                None => {}
                Some(downloads) => {
                    match downloads.artifact {
                        None => {}
                        Some(artifact) => match path::get_library_path(&artifact.path) {
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
                                                    Ok(yay) => {}
                                                    Err(ohno) => {
                                                        result = None;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        Err(meta_err) => {
                                            result = None;
                                            break;
                                        }
                                    }
                                } else {
                                    match path::download_file_to(&artifact.url, &lib_path) {
                                        Ok(yay) => {}
                                        Err(ohno) => {
                                            result = None;
                                            break;
                                        }
                                    }
                                }
                            }
                        },
                    }

                    match downloads.classifiers {
                        None => {}
                        Some(class) => {
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
                                                match path::download_file_to(&class.url, &lib_path)
                                                {
                                                    Ok(yay) => {}
                                                    Err(ohno) => {
                                                        result = None;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        Err(meta_err) => {
                                            result = None;
                                            break;
                                        }
                                    }
                                } else {
                                    match path::download_file_to(&class.url, &lib_path) {
                                        Ok(yay) => {}
                                        Err(ohno) => {
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

            match library.extract {
                None => {}
                Some(_) => {
                    println!("Library extraction is not handled yet, sorry :/")
                }
            }

            match library.url {
                None => {}
                Some(url) => {
                    match path::get_library_path(
                        &(group.replace(".", "/")
                            + "/"
                            + name
                            + "/"
                            + name
                            + "-"
                            + version
                            + ".jar"),
                    ) {
                        None => {
                            result = None;
                            break;
                        }
                        Some(lib_path) => {
                            if !lib_path.exists() {
                                match path::download_file_to(&url, &lib_path) {
                                    Ok(yay) => {}
                                    Err(ohno) => {
                                        result = None;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

fn install_assets_index(version_manifest: &version::Main) -> Option<()> {
    let version_manifest = version_manifest.clone();

    match version_manifest.asset_index {
        None => None,
        Some(a_index) => match path::get_assets_folder(&String::from("indexes")) {
            None => None,
            Some(index_folder) => {
                let index_file = index_folder.join(format!("{}.json", &a_index.id));

                if index_file.exists() {
                    match index_file.metadata() {
                        Ok(meta) => {
                            if meta.len() != a_index.size {
                                match path::download_file_to(&a_index.url, &index_file) {
                                    Ok(msg) => update_assets(a_index.id),
                                    Err(err_msg) => None,
                                }
                            } else {
                                Some(())
                            }
                        }
                        Err(err) => None,
                    }
                } else {
                    match path::download_file_to(&a_index.url, &index_file) {
                        Ok(msg) => update_assets(a_index.id),
                        Err(err_msg) => None,
                    }
                }
            }
        },
    }
}

fn update_assets(index: String) -> Option<()> {
    match path::get_assets_folder(&String::from("indexes")) {
        None => None,
        Some(index_folder) => {
            let index_file = index_folder.join(format!("{}.json", index));

            if index_file.exists() {
                match File::open(index_file) {
                    Ok(mut index_file) => {
                        let mut body = String::new();
                        match index_file.read_to_string(&mut body) {
                            Ok(_) => {}
                            Err(_) => return None,
                        };

                        match assets::parse(&body) {
                            Ok(main) => match path::get_assets_folder(&String::from("objects")) {
                                None => None,
                                Some(object_path) => {
                                    let mut ret = Some(());
                                    for entry in main.objects {
                                        let asset_path = entry.1.get_download_path(&object_path);

                                        if asset_path.exists() {
                                            match asset_path.metadata() {
                                                Ok(meta) => {
                                                    if meta.len() != entry.1.size {
                                                        match path::download_file_to(
                                                            &entry.1.get_download_url(),
                                                            &asset_path,
                                                        ) {
                                                            Ok(msg) => {}
                                                            Err(err) => {
                                                                ret = None;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    ret = None;
                                                    break;
                                                }
                                            }
                                        } else {
                                            match path::download_file_to(
                                                &entry.1.get_download_url(),
                                                &asset_path,
                                            ) {
                                                Ok(msg) => {}
                                                Err(err) => {
                                                    ret = None;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    ret
                                }
                            },
                            Err(err) => None,
                        }
                    }
                    Err(err) => None,
                }
            } else {
                None
            }
        }
    }
}

fn check_java_version(version_manifest: &version::Main) -> Option<()> {
    let version_manifest = version_manifest.clone();
    match get_java_version_manifest() {
        None => {
            match path::get_java_folder_path(match version_manifest.java_version {
                None => &String::from("jre-legacy"),
                Some(java_v) => &java_v.component
            }) {
                None => None,
                Some(java_folder) => if &java_folder.exists() {
                    match path::get_or_create_dirs(&java_folder, get_java_folder_for_os()) {
                        None => None,
                        Some(bin) => if bin.join(get_java_ex_for_os()).exists() {
                            Some(())
                        } else { None }
                    }
                } else { None }
            }
        }

        Some(manifest) => {

        }
    }
}

fn get_java_folder_for_os() -> Vec<String> {
    match std::env::consts::OS {
        "macos" => vec![String::from("jre.bundle"), String::from("Contents"), String::from("Home"), String::from("bin")],
        &_ => vec![String::from("bin")]
    }
}

fn get_java_ex_for_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "java.exe",
        &_ => "java"
    }
}

fn get_java_version_manifest() -> Option<java_versions::Main> {
    match path::read_file_from_url_to_string(&"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json".to_string()) {
        Ok(body) => {
            match java_versions::parse_java_versions_manifest(&body) {
                Ok(manifest) => Some(manifest),
                Err(err) => {
                    print!("Error: {}", err.to_string());
                    None
                }
            }
        }
        Err(err) => {
            print!("Error: {}", err);
            None
        }
    }
}
