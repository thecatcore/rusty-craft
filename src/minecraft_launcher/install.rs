use crate::minecraft_launcher::{
    arguments,
    manifest::assets,
    manifest::assets::Main,
    manifest::java,
    manifest::java_versions,
    manifest::version,
    manifest::version::{
        AssetIndex, Downloads, LibraryDownloadArtifact, LibraryExtract, Rule, RuleAction,
    },
    path,
};

use crate::minecraft_launcher::manifest;
use crate::minecraft_launcher::manifest::java_versions::{OsVersions, Version};
use crate::minecraft_launcher::manifest::version::{JavaVersion, Logging, ClientLogging};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{Error, Read, Write};
use std::path::PathBuf;

use std::os::unix::fs::PermissionsExt;

pub fn install_version(id: String, versions: Vec<manifest::main::Version>) -> Option<()> {
    match path::get_version_folder(&id) {
        None => None,
        Some(version_folder) => {
            let manifest_file_path = version_folder.join(id.clone() + ".json");
            if manifest_file_path.exists() {
                read_version_manifest(manifest_file_path)
            } else {
                for version in versions {
                    if version.id == id {
                        return install_manifest(version, manifest_file_path);
                    }
                }
                None
            }
        }
    }
}

fn install_manifest(version: manifest::main::Version, file_path: PathBuf) -> Option<()> {
    match path::download_file_to(&version.url, &file_path) {
        Ok(_) => read_version_manifest(file_path),
        Err(_) => None,
    }
}

fn read_version_manifest(manifest_path: PathBuf) -> Option<()> {
    match File::open(manifest_path) {
        Ok(mut file) => {
            let mut body = String::new();
            match file.read_to_string(&mut body) {
                Ok(_) => match manifest::version::parse_version_manifest(&body) {
                    Ok(version) => install_version_from_manifest(&version),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn install_version_from_manifest(version_manifest: &version::Main) -> Option<()> {
    println!("Checking java");
    match check_java_version(version_manifest) {
        None => {
            return None;
        }
        Some(_) => {}
    }

    println!("Checking client jar");
    match install_client_jar(version_manifest) {
        None => {
            return None;
        }
        Some(_) => {}
    }

    println!("Checking libraries");
    match install_libraries(version_manifest) {
        None => {
            return None;
        }
        Some(_) => {}
    }

    println!("Checking assets");
    match install_assets_index(version_manifest) {
        None => {
            return None;
        }
        Some(_) => {}
    }

    println!("Checking log file");
    match check_log_file(version_manifest) {
        None => {
            return None;
        }
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
                Some(extract) => {
                    println!("Library extraction is not handled yet, sorry :/")
                }
            }

            match library.url {
                None => {}
                Some(url) => {
                    let url_path =
                        group.replace(".", "/") + "/" + name + "/" + name + "-" + version + ".jar";
                    match path::get_library_path(&url_path) {
                        None => {
                            result = None;
                            break;
                        }
                        Some(lib_path) => {
                            if !lib_path.exists() {
                                match path::download_file_to(&(url + "/" + &*url_path), &lib_path) {
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
        None => {
            println!("Version manifest doesn't contain any asset index!");
            None
        },
        Some(a_index) => {
            println!("Got asset index");
            match path::get_assets_folder(&String::from("indexes")) {
                None => {
                    println!("Unable to get indexes folder");
                    None
                },
                Some(index_folder) => {
                    println!("Got indexes folder");
                    let index_file = index_folder.join(format!("{}.json", &a_index.id));

                    if index_file.exists() {
                        println!("Asset index file {}.json exists", &a_index.id);
                        match index_file.metadata() {
                            Ok(meta) => {
                                println!("Read asset index meta");
                                if meta.len() != a_index.size {
                                    println!("Different size detected!");
                                    match path::download_file_to(&a_index.url, &index_file) {
                                        Ok(msg) => {
                                            println!("Successfully downloaded new index");
                                            update_assets(a_index.id)
                                        },
                                        Err(err_msg) => {
                                            println!("Unable to download asset index file: {}", err_msg);
                                            None
                                        },
                                    }
                                } else {
                                    println!("Size is the same, checking assets one by one");
                                    update_assets(a_index.id)
                                }
                            }
                            Err(err) => {
                                println!("Can't access meta attempting to redownload the file");
                                match path::download_file_to(&a_index.url, &index_file) {
                                    Ok(msg) => {
                                        println!("Successfully downloaded index file");
                                        update_assets(a_index.id)
                                    },
                                    Err(err_msg) => {
                                        println!("Unable to download asset index file: {}", err_msg);
                                        None
                                    },
                                }
                            },
                        }
                    } else {
                        println!("Asset index file {}.json doesn't exist", &a_index.id);
                        match path::download_file_to(&a_index.url, &index_file) {
                            Ok(msg) => {
                                println!("Successfully downloaded index file");
                                update_assets(a_index.id)
                            },
                            Err(err_msg) => {
                                println!("Unable to download asset index file: {}", err_msg);
                                None
                            },
                        }
                    }
                }
            }
        },
    }
}

fn update_assets(index: String) -> Option<()> {
    match path::get_assets_folder(&String::from("indexes")) {
        None => {
            println!("Unable to get indexes folder");
            None
        },
        Some(index_folder) => {
            println!("Got indexes folder");
            let index_file = index_folder.join(format!("{}.json", index));

            if index_file.exists() {
                println!("Asset index file exists");
                match File::open(index_file) {
                    Ok(mut index_file) => {
                        println!("Opened index file");
                        let mut body = String::new();
                        match index_file.read_to_string(&mut body) {
                            Ok(_) => {
                                println!("Read index file");
                                match assets::parse(&body) {
                                    Ok(main) => {
                                        println!("Parsed index file");
                                        match path::get_assets_folder(&String::from("objects")) {
                                            None => {
                                                println!("Unable to get objects folder");
                                                None
                                            },
                                            Some(object_path) => {
                                                println!("Got objects folder");
                                                let mut ret = Some(());
                                                for entry in main.objects {
                                                    let asset_path = entry.1.get_download_path(&object_path);

                                                    if asset_path.1.exists() {
                                                        match asset_path.1.metadata() {
                                                            Ok(meta) => {
                                                                if meta.len() != entry.1.size {
                                                                    match path::download_file_to(
                                                                        &entry.1.get_download_url(),
                                                                        &asset_path.1,
                                                                    ) {
                                                                        Ok(msg) => {}
                                                                        Err(err) => {
                                                                            println!("Unable to download file {}: {}", entry.0, err);
                                                                            ret = None;
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(err) => {
                                                                match path::get_or_create_dir(&object_path, asset_path.0) {
                                                                    None => {
                                                                        println!("Unable to create folder for asset");
                                                                        ret = None;
                                                                        break;
                                                                    }
                                                                    Some(_) => {
                                                                        match path::download_file_to(
                                                                            &entry.1.get_download_url(),
                                                                            &asset_path.1,
                                                                        ) {
                                                                            Ok(msg) => {}
                                                                            Err(err) => {
                                                                                println!("Unable to download file {}: {}", &entry.0, err);
                                                                                ret = None;
                                                                                break;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        match path::get_or_create_dir(&object_path, asset_path.0) {
                                                            None => {
                                                                println!("Unable to create folder for asset");
                                                                ret = None;
                                                                break;
                                                            }
                                                            Some(_) => {
                                                                match path::download_file_to(
                                                                    &entry.1.get_download_url(),
                                                                    &asset_path.1,
                                                                ) {
                                                                    Ok(msg) => {}
                                                                    Err(err) => {
                                                                        println!("Unable to download file {}: {}", &entry.0, err);
                                                                        ret = None;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                ret
                                            }
                                        }
                                    },
                                    Err(err) => {
                                        println!("Unable to parsed index file: {}", err);
                                        None
                                    },
                                }
                            }
                            Err(err) => {
                                println!("Unable to read index file: {}", err);
                                None
                            },
                        }
                    }
                    Err(err) => {
                        println!("Unable to opened index file: {}", err);
                        None
                    },
                }
            } else {
                println!("Asset index file doesn't exist");
                None
            }
        }
    }
}

fn check_java_version(version_manifest: &version::Main) -> Option<()> {
    let version_manifest = version_manifest.clone();
    match get_java_version_manifest() {
        None => {
            println!("Can't get java versions manifest");
            match path::get_java_folder_path_sub(
                &(match version_manifest.java_version {
                    None => {
                        println!("Using default java version");
                        String::from("jre-legacy")
                    }
                    Some(java_v) => {
                        println!("Found java version {}", java_v.component);
                        java_v.component
                    }
                }),
            ) {
                None => {
                    println!("Can't get java_folder_path_sub");
                    None
                }
                Some(java_folder) => {
                    match path::get_or_create_dirs(&java_folder, get_java_folder_for_os()) {
                        None => None,
                        Some(bin) => {
                            if (&java_folder).exists() {
                                if bin.join(get_java_ex_for_os()).exists() {
                                    Some(())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
            }
        }

        Some(manifest) => {
            println!("Got java versions manifest");
            match manifest.get_os_version() {
                None => {
                    println!("Unable to get os_version");
                    None
                }
                Some(os_version) => {
                    println!("Got os_version");
                    let java_v_type = match version_manifest.java_version {
                        None => {
                            println!("Using default java version");
                            String::from("jre-legacy")
                        }
                        Some(ver) => {
                            println!("Found java version {}", ver.component);
                            ver.component
                        }
                    };
                    match os_version.get_java_version(&java_v_type) {
                        None => {
                            println!("Unable to get java_version");
                            None
                        }
                        Some(versions) => {
                            println!("Got java_version");
                            match versions.get(0) {
                                None => {
                                    println!("Unable to get first version");
                                    None
                                }
                                Some(version) => {
                                    println!("Got first version");
                                    let online_version = version.clone().version.name;
                                    println!("Found online version of java {}", online_version);
                                    match path::get_java_folder_path_sub(&java_v_type) {
                                        None => {
                                            println!("Unable to get java_folder_path_sub");
                                            None
                                        }
                                        Some(j_folder) => {
                                            println!("Got java_folder_path_sub");
                                            match path::get_java_folder_path(&java_v_type) {
                                                None => {
                                                    println!("Unable to get java_folder_path");
                                                    None
                                                }
                                                Some(os_fol) => {
                                                    println!("Got java_folder_path");
                                                    if (&j_folder).exists() {
                                                        println!("java_folder_path_sub exists");
                                                        match File::open(os_fol.join(".version")) {
                                                            Ok(mut v_file) => {
                                                                println!("Opened .version file");
                                                                let mut v_content = String::new();
                                                                match v_file
                                                                    .read_to_string(&mut v_content)
                                                                {
                                                                    Ok(_) => {
                                                                        println!(
                                                                            "Read .version file"
                                                                        );
                                                                        if online_version
                                                                            != v_content
                                                                        {
                                                                            println!("Online and local version aren't the same");
                                                                            match install_java_version(&java_v_type, os_fol, version.clone().manifest, online_version) {
                                                                                None => None,
                                                                                Some(_) => Some(())
                                                                            }
                                                                        } else {
                                                                            Some(())
                                                                        }
                                                                    }
                                                                    Err(_) => {
                                                                        println!("Failed to read .version file");
                                                                        match install_java_version(
                                                                            &java_v_type,
                                                                            os_fol,
                                                                            version
                                                                                .clone()
                                                                                .manifest,
                                                                            online_version,
                                                                        ) {
                                                                            None => None,
                                                                            Some(_) => Some(()),
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(_) => {
                                                                println!("Failed to opened .version file");
                                                                match install_java_version(
                                                                    &java_v_type,
                                                                    os_fol,
                                                                    version.clone().manifest,
                                                                    online_version,
                                                                ) {
                                                                    None => None,
                                                                    Some(_) => Some(()),
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        println!(
                                                            "java_folder_path_sub doesn't exists"
                                                        );
                                                        match install_java_version(
                                                            &java_v_type,
                                                            os_fol,
                                                            version.clone().manifest,
                                                            online_version,
                                                        ) {
                                                            None => None,
                                                            Some(_) => Some(()),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn install_java_version(
    type_: &String,
    os_folder: PathBuf,
    manifest: java_versions::Manifest,
    online_version: String,
) -> Option<()> {
    let v_folder = match path::get_or_create_dir(&os_folder, type_.clone()) {
        None => {
            println!("Failed to get v_folder");
            os_folder.clone()
        }
        Some(v) => {
            println!("Got v_folder");
            v
        }
    };
    match path::read_file_from_url_to_string(&manifest.url) {
        Ok(stri) => {
            println!("Read java_version_manifest");
            match java::parse_java_version_manifest(&stri) {
                Ok(manifest) => {
                    println!("Parsed java_version_manifest");
                    let mut status: Option<()> = Some(());
                    for file in manifest.files {
                        if status.is_none() {
                            break;
                        }
                        let file_path = file.0;
                        let element_info = file.1;
                        let el_type = element_info.element_type;
                        let executable = match element_info.executable {
                            None => false,
                            Some(bool) => bool,
                        };
                        if el_type == "directory" {
                            if file_path.contains("/") {
                                let parts: Vec<&str> = file_path.split("/").collect();
                                let mut parts2: Vec<String> = Vec::new();
                                for part in parts {
                                    parts2.push(part.to_string());
                                }
                                let parts = parts2;
                                status = match path::get_or_create_dirs(&v_folder, parts) {
                                    None => None,
                                    Some(_) => Some(()),
                                }
                            } else {
                                status = match path::get_or_create_dir(&v_folder, file_path) {
                                    None => None,
                                    Some(_) => Some(()),
                                }
                            }
                        } else if el_type == "file" {
                            status = match element_info.downloads {
                                None => {
                                    println!("Failed to get download for file {}", file_path);
                                    None
                                }
                                Some(downloads) => {
                                    println!("Got download for file {}", file_path);
                                    let url = downloads.raw.url;
                                    if file_path.contains("/") {
                                        println!("File path contains '/'");
                                        let parts: Vec<&str> = file_path.split("/").collect();
                                        let mut parts2: Vec<String> = Vec::new();
                                        for part in parts {
                                            parts2.push(part.to_string());
                                        }
                                        match parts2.split_last() {
                                            None => {
                                                println!("Unable to split_last {}", file_path);
                                                None
                                            }
                                            Some(tuple) => {
                                                println!("Split_lasted {}", file_path);
                                                let parts = Vec::from(tuple.1);
                                                match path::get_or_create_dirs(&v_folder, parts) {
                                                    None => {
                                                        println!("Unable to create folders");
                                                        None
                                                    }
                                                    Some(sub_pathh) => {
                                                        println!("Created folders");
                                                        let file_buf = sub_pathh.join(tuple.0);
                                                        match path::download_file_to(
                                                            &url, &file_buf,
                                                        ) {
                                                            Ok(_) => {
                                                                println!(
                                                                    "Successfully downloaded file!"
                                                                );
                                                                if executable {
                                                                    match std::env::consts::OS {
                                                                        "linux" => {
                                                                            match file_buf
                                                                                .metadata()
                                                                            {
                                                                                Ok(meta) => {
                                                                                    let mut perm = meta.permissions();
                                                                                    perm.set_mode(
                                                                                        0o755,
                                                                                    );
                                                                                    match std::fs::set_permissions(file_buf, perm) {
                                                                                        Ok(_) => Some(()),
                                                                                        Err(_) => None
                                                                                    }
                                                                                }
                                                                                Err(_) => None,
                                                                            }
                                                                        }
                                                                        &_ => None,
                                                                    }
                                                                } else {
                                                                    Some(())
                                                                }
                                                            }
                                                            Err(err) => {
                                                                println!(
                                                                    "Failed to download file: {}",
                                                                    err
                                                                );
                                                                None
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        println!("File path doesn't contain '/'");
                                        let file_buf = v_folder.join(file_path);
                                        match path::download_file_to(&url, &file_buf) {
                                            Ok(_) => {
                                                println!("Successfully downloaded file");
                                                if executable {
                                                    match std::env::consts::OS {
                                                        "linux" => match file_buf.metadata() {
                                                            Ok(meta) => {
                                                                let mut perm = meta.permissions();
                                                                perm.set_mode(0o755);
                                                                match std::fs::set_permissions(
                                                                    file_buf, perm,
                                                                ) {
                                                                    Ok(_) => Some(()),
                                                                    Err(_) => None,
                                                                }
                                                            }
                                                            Err(_) => None,
                                                        },
                                                        &_ => None,
                                                    }
                                                } else {
                                                    Some(())
                                                }
                                            }
                                            Err(err) => {
                                                println!("Failed to download file: {}", err);
                                                None
                                            }
                                        }
                                    }
                                }
                            };
                        } else if el_type == "link" {

                        } else {
                            println!("Unknown el_type {}", el_type);
                        }
                    }
                    if status.is_some() {
                        let v_path = os_folder.join(".version");
                        match File::open(&v_path) {
                            Ok(mut v_path) => match v_path.write(online_version.as_bytes()) {
                                Ok(_) => {
                                    println!("Wrote to .version file")
                                }
                                Err(_) => {
                                    println!("Failed to write to .version file");
                                    status = None
                                }
                            },
                            Err(_) => match File::create(v_path) {
                                Ok(mut v_path) => match v_path.write(online_version.as_bytes()) {
                                    Ok(_) => {
                                        println!("Wrote to .version file")
                                    }
                                    Err(_) => {
                                        println!("Failed to write to .version file");
                                        status = None
                                    }
                                },
                                Err(err) => {
                                    println!("Failed to create .version file: {}", err);
                                    status = None;
                                }
                            },
                        }
                    }
                    status
                }
                Err(err) => {
                    println!("Failed to parse java_version_manifest {}", err);
                    None
                }
            }
        }
        Err(err) => {
            println!("Failed to read java_version_manifest {}", err);
            None
        }
    }
}

fn get_java_folder_for_os() -> Vec<String> {
    match std::env::consts::OS {
        "macos" => vec![
            String::from("jre.bundle"),
            String::from("Contents"),
            String::from("Home"),
            String::from("bin"),
        ],
        &_ => vec![String::from("bin")],
    }
}

fn get_java_ex_for_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "java.exe",
        &_ => "java",
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

fn check_log_file(version_manifest: &version::Main) -> Option<()> {
    let version_manifest = version_manifest.clone();
    match version_manifest.logging {
        None => {
            println!("No logging, that's fine");
            Some(())
        }
        Some(logging) => {
            match logging.client {
                None => {
                    println!("No logging (2), that's fine");
                    Some(())
                }
                Some(client_log) => {
                    let file_info = client_log.file;
                    match path::get_assets_folder(&String::from("log_configs")) {
                        None => {
                            println!("Unable to get log_configs folder");
                            None
                        }
                        Some(log_folder) => {
                            let log_path = log_folder.join(file_info.id);
                            if log_path.exists() {
                                match log_path.metadata() {
                                    Ok(meta) => {
                                        if meta.len() != file_info.size {
                                            match path::download_file_to(&file_info.url, &log_path) {
                                                Ok(_) => {
                                                    Some(())
                                                }
                                                Err(err) => {
                                                    println!("Unable to download logger file: {}", err);
                                                    None
                                                }
                                            }
                                        } else {
                                            Some(())
                                        }
                                    }
                                    Err(_) => {
                                        match path::download_file_to(&file_info.url, &log_path) {
                                            Ok(_) => {
                                                Some(())
                                            }
                                            Err(err) => {
                                                println!("Unable to download logger file: {}", err);
                                                None
                                            }
                                        }
                                    }
                                }
                            } else {
                                match path::download_file_to(&file_info.url, &log_path) {
                                    Ok(_) => {
                                        Some(())
                                    }
                                    Err(err) => {
                                        println!("Unable to download logger file: {}", err);
                                        None
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}