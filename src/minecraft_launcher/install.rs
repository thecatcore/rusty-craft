use crate::minecraft_launcher::{
    arguments,
    manifest::assets,
    manifest::version,
    manifest::version::{LibraryDownloadArtifact, RuleAction},
    path,
    launch,
};

use crate::minecraft_launcher::app::download_tab::Message;
use crate::minecraft_launcher::manifest;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

mod java;

pub fn install_version(
    id: String,
    versions: Vec<manifest::main::Version>,
    tx: Sender<Message>,
) -> Option<()> {
    tx.send(Message::NewSubStep(
        String::from("Checking Version folder"),
        1,
        3,
    ))
    .expect("Can't send message to renderer thread");
    match path::get_version_folder(&id) {
        None => None,
        Some(version_folder) => {
            let manifest_file_path = version_folder.join(id.clone() + ".json");
            if manifest_file_path.exists() {
                read_version_manifest(manifest_file_path, tx)
            } else {
                for version in versions {
                    if version.id == id {
                        return install_manifest(version, manifest_file_path, tx);
                    }
                }
                None
            }
        }
    }
}

fn install_manifest(
    version: manifest::main::Version,
    file_path: PathBuf,
    tx: Sender<Message>,
) -> Option<()> {
    tx.send(Message::NewSubStep(
        String::from("Downloading Version manifest"),
        2,
        3,
    ))
    .expect("Can't send message to renderer thread");
    match path::download_file_to(&version.url, &file_path) {
        Ok(_) => read_version_manifest(file_path, tx),
        Err(_) => None,
    }
}

fn read_version_manifest(manifest_path: PathBuf, tx: Sender<Message>) -> Option<()> {
    tx.send(Message::NewSubStep(
        String::from("Reading Version manifest"),
        3,
        3,
    ))
    .expect("Can't send message to renderer thread");
    match File::open(manifest_path) {
        Ok(mut file) => {
            let mut body = String::new();
            match file.read_to_string(&mut body) {
                Ok(_) => match manifest::version::parse_version_manifest(&body) {
                    Ok(version) => install_version_from_manifest(&version, tx),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn install_version_from_manifest(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<()> {
    // println!("Checking java");
    match java::check_java_version(version_manifest, tx) {
        None => None,
        Some(tx) => {
            // println!("Checking client jar");
            match install_client_jar(version_manifest, tx) {
                None => None,
                Some(tx) => {
                    // println!("Checking libraries");
                    match install_libraries(version_manifest, tx) {
                        None => None,
                        Some(tx) => {
                            // println!("Checking assets");
                            match install_assets_index(version_manifest, tx) {
                                None => None,
                                Some(tx) => {
                                    // println!("Checking log file");
                                    match check_log_file(version_manifest, tx) {
                                        None => None,
                                        Some(tx) => {
                                            launch::pre_launch(version_manifest.clone(), tx);
                                            Some(())
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

fn install_client_jar(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();
    tx.send(Message::NewStep(3))
        .expect("Can't send message to renderer thread");
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
                                        Ok(_) => Some(tx),
                                        Err(err) => {
                                            println!("{}", err);
                                            None
                                        }
                                    }
                                } else {
                                    Some(tx)
                                }
                            }
                            Err(err) => {
                                println!("Error while trying to get client jar metadata: {}", err);
                                None
                            }
                        }
                    } else {
                        match path::download_file_to(&client_entry.url, &jar_path) {
                            Ok(_) => Some(tx),
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

fn install_libraries(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();

    tx.send(Message::NewStep(4))
        .expect("Can't send message to renderer thread");
    let mut result = Some(());

    let library_count = version_manifest.libraries.len();

    let mut index = 0;

    for library in version_manifest.libraries {
        index += 1;
        let lib_name: Vec<&str> = library.name.split(":").collect();
        let group = *lib_name.get(0).expect("Library doesn't have a group???");
        let name = *lib_name.get(1).expect("Library doesn't have a name???");
        let version = *lib_name.get(2).expect("Library doesn't have a version???");

        tx.send(Message::NewSubStep(
            format!("{}-{}", &name, &version),
            index,
            library_count as u64,
        ))
        .expect("Can't send message to renderer thread");

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
                                                                println!("{}", err);
                                                                result = None;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(meta_err) => {
                                                    println!("{}", meta_err);
                                                    result = None;
                                                    break;
                                                }
                                            }
                                        } else {
                                            match path::download_file_to(&artifact.url, &lib_path) {
                                                Ok(_) => {}
                                                Err(ohno) => {
                                                    println!("{}", ohno);
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
                                                        Err(ohno) => {
                                                            println!("{}", ohno);
                                                            result = None;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(meta_err) => {
                                                println!("{}", meta_err);
                                                result = None;
                                                break;
                                            }
                                        }
                                    } else {
                                        match path::download_file_to(&class.url, &lib_path) {
                                            Ok(_) => {}
                                            Err(ohno) => {
                                                println!("{}", ohno);
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
                                            println!("{}", ohno);
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
                        group.replace(".", "/") + "/" + name + "/" + name + "-" + version + ".jar";
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
                                    Err(ohno) => {
                                        println!("{}", ohno);
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

fn install_assets_index(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();

    tx.send(Message::NewStep(5))
        .expect("Can't send message to renderer thread");
    tx.send(Message::NewSubStep(format!("Checking asset index"), 1, 3))
        .expect("Can't send message to renderer thread");
    match version_manifest.asset_index {
        None => {
            println!("Version manifest doesn't contain any asset index!");
            None
        }
        Some(a_index) => {
            // println!("Got asset index");
            match path::get_assets_folder(&String::from("indexes")) {
                None => {
                    println!("Unable to get indexes folder");
                    None
                }
                Some(index_folder) => {
                    // println!("Got indexes folder");
                    let index_file = index_folder.join(format!("{}.json", &a_index.id));

                    if index_file.exists() {
                        // println!("Asset index file {}.json exists", &a_index.id);
                        match index_file.metadata() {
                            Ok(meta) => {
                                // println!("Read asset index meta");
                                if meta.len() != a_index.size {
                                    // println!("Different size detected!");
                                    match path::download_file_to(&a_index.url, &index_file) {
                                        Ok(_) => {
                                            // println!("Successfully downloaded new index");
                                            update_assets(a_index.id, tx)
                                        }
                                        Err(err_msg) => {
                                            println!(
                                                "Unable to download asset index file: {}",
                                                err_msg
                                            );
                                            None
                                        }
                                    }
                                } else {
                                    // println!("Size is the same, checking assets one by one");
                                    update_assets(a_index.id, tx)
                                }
                            }
                            Err(_) => {
                                // println!("Can't access meta attempting to redownload the file");
                                match path::download_file_to(&a_index.url, &index_file) {
                                    Ok(_) => {
                                        // println!("Successfully downloaded index file");
                                        update_assets(a_index.id, tx)
                                    }
                                    Err(err_msg) => {
                                        println!(
                                            "Unable to download asset index file: {}",
                                            err_msg
                                        );
                                        None
                                    }
                                }
                            }
                        }
                    } else {
                        // println!("Asset index file {}.json doesn't exist", &a_index.id);
                        match path::download_file_to(&a_index.url, &index_file) {
                            Ok(_) => {
                                // println!("Successfully downloaded index file");
                                update_assets(a_index.id, tx)
                            }
                            Err(err_msg) => {
                                println!("Unable to download asset index file: {}", err_msg);
                                None
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_assets(index: String, tx: Sender<Message>) -> Option<Sender<Message>> {
    tx.send(Message::NewSubStep(
        format!("Installing missing assets"),
        2,
        3,
    ))
    .expect("Can't send message to renderer thread");
    match path::get_assets_folder(&String::from("indexes")) {
        None => {
            println!("Unable to get indexes folder");
            None
        }
        Some(index_folder) => {
            // println!("Got indexes folder");
            let index_file = index_folder.join(format!("{}.json", index));

            if index_file.exists() {
                // println!("Asset index file exists");
                match File::open(index_file) {
                    Ok(mut index_file) => {
                        // println!("Opened index file");
                        let mut body = String::new();
                        match index_file.read_to_string(&mut body) {
                            Ok(_) => {
                                // println!("Read index file");
                                match assets::parse(&body) {
                                    Ok(main) => {
                                        // println!("Parsed index file");
                                        let result = match path::get_assets_folder(&String::from(
                                            "objects",
                                        )) {
                                            None => {
                                                println!("Unable to get objects folder");
                                                None
                                            }
                                            Some(object_path) => {
                                                // println!("Got objects folder");
                                                let mut ret = Some(());
                                                let entry_count = main.objects.len();
                                                let mut entry_index = 0;
                                                for entry in main.objects.clone() {
                                                    entry_index += 1;

                                                    tx.send(Message::NewSubSubStep(format!("{}", entry.0.clone()), entry_index, entry_count as u64)).expect("Can't send message to renderer thread");

                                                    let asset_path =
                                                        entry.1.get_download_path(&object_path);

                                                    if asset_path.1.exists() {
                                                        match asset_path.1.metadata() {
                                                            Ok(meta) => {
                                                                if meta.len() != entry.1.size {
                                                                    match path::download_file_to(
                                                                        &entry.1.get_download_url(),
                                                                        &asset_path.1,
                                                                    ) {
                                                                        Ok(_) => {}
                                                                        Err(err) => {
                                                                            println!("Unable to download file {}: {}", entry.0, err);
                                                                            ret = None;
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(_) => {
                                                                match path::get_or_create_dir(
                                                                    &object_path,
                                                                    asset_path.0,
                                                                ) {
                                                                    None => {
                                                                        println!("Unable to create folder for asset");
                                                                        ret = None;
                                                                        break;
                                                                    }
                                                                    Some(_) => {
                                                                        match path::download_file_to(
                                                                            &entry
                                                                                .1
                                                                                .get_download_url(),
                                                                            &asset_path.1,
                                                                        ) {
                                                                            Ok(_) => {}
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
                                                        match path::get_or_create_dir(
                                                            &object_path,
                                                            asset_path.0,
                                                        ) {
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
                                                                    Ok(_) => {}
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
                                                match ret {
                                                    None => None,
                                                    Some(_) => Some(tx),
                                                }
                                            }
                                        };

                                        match result {
                                            None => None,
                                            Some(tx) => {
                                                match main.map_to_resources {
                                                    None => Some(tx),
                                                    Some(map_to_resources) => {
                                                        if map_to_resources {
                                                            tx.send(Message::NewSubStep(format !("Relocating to resources folder"), 3, 3))
                                                                .expect("Can't send message to renderer thread");
                                                            match path::get_minecraft_sub_folder(
                                                                &String::from("resources"),
                                                            ) {
                                                                None => None,
                                                                Some(resources) => {
                                                                    match path::get_assets_folder(
                                                                        &String::from("objects"),
                                                                    ) {
                                                                        None => None,
                                                                        Some(objects_path) => {
                                                                            let entry_count =
                                                                                main.objects.len();
                                                                            let mut entry_index = 0;
                                                                            let mut res = Some(());
                                                                            for (
                                                                                entry,
                                                                                asset_info,
                                                                            ) in main.objects
                                                                            {
                                                                                entry_index += 1;
                                                                                tx.send(Message::NewSubSubStep(format!("{}", entry.clone()), entry_index, entry_count as u64)).expect("Can't send message to renderer thread");
                                                                                let hashed_path = asset_info.get_download_path(&objects_path);
                                                                                if hashed_path
                                                                                    .1
                                                                                    .exists()
                                                                                {
                                                                                    match File::open(
                                                                                        hashed_path
                                                                                            .1,
                                                                                    ) {
                                                                                        Ok(
                                                                                            mut
                                                                                            file,
                                                                                        ) => {
                                                                                            let mut body: Vec<u8> = Vec::new();
                                                                                            match file.read_to_end(&mut body) {
                                                                                                Ok(_) => {
                                                                                                    if entry.contains("/") {
                                                                                                        // let parts: Vec<&str> = entry.split("/").collect();
                                                                                                        // let mut parts2: Vec<String> = Vec::new();
                                                                                                        // for part in parts {
                                                                                                        //     parts2.push(part.to_string());
                                                                                                        // }
                                                                                                        let entry_pathbuf = PathBuf::from(entry);

                                                                                                        match path::get_or_create_dir(&resources, String::from(entry_pathbuf.parent().unwrap().to_str().unwrap())) {
                                                                                                            None => {
                                                                                                                println!("Unable to get asset path");
                                                                                                                res = None;
                                                                                                                break;
                                                                                                            }
                                                                                                            Some(file_path) => {
                                                                                                                let file_path = file_path.join(entry_pathbuf.components().last().unwrap());
                                                                                                                match File::create(file_path) {
                                                                                                                    Ok(mut file) => {
                                                                                                                        match file.write(body.as_slice()) {
                                                                                                                            Ok(_) => {}
                                                                                                                            Err(err) => {
                                                                                                                                println!("Unable to write to asset file: {}", err);
                                                                                                                                res = None;
                                                                                                                                break;
                                                                                                                            }
                                                                                                                        }
                                                                                                                    }
                                                                                                                    Err(err) => {
                                                                                                                        println!("Unable to create asset file: {}", err);
                                                                                                                        res = None;
                                                                                                                        break;
                                                                                                                    }
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                    } else {
                                                                                                        let resource_path = resources.join(entry);
                                                                                                        match File::create(resource_path) {
                                                                                                            Ok(mut file) => {
                                                                                                                match file.write(body.as_slice()) {
                                                                                                                    Ok(_) => {}
                                                                                                                    Err(err) => {
                                                                                                                        println!("Unable to write to asset file: {}", err);
                                                                                                                        res = None;
                                                                                                                        break;
                                                                                                                    }
                                                                                                                }
                                                                                                            }
                                                                                                            Err(err) => {
                                                                                                                println!("Unable to create asset file: {}", err);
                                                                                                                res = None;
                                                                                                                break;
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                                Err(err) => {
                                                                                                    println!("Unable to read asset file: {}", err);
                                                                                                    res = None;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        Err(
                                                                                            err,
                                                                                        ) => {
                                                                                            println!("Unable to open asset file: {}", err);
                                                                                            res = None;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                } else {
                                                                                    res = None;
                                                                                    break;
                                                                                }
                                                                            }

                                                                            match res {
                                                                                None => None,
                                                                                Some(_) => Some(tx),
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            Some(tx)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("Unable to parsed index file: {}", err);
                                        None
                                    }
                                }
                            }
                            Err(err) => {
                                println!("Unable to read index file: {}", err);
                                None
                            }
                        }
                    }
                    Err(err) => {
                        println!("Unable to opened index file: {}", err);
                        None
                    }
                }
            } else {
                println!("Asset index file doesn't exist");
                None
            }
        }
    }
}

fn check_log_file(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();
    tx.send(Message::NewStep(6))
        .expect("Can't send message to renderer thread");
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
                                            Ok(_) => Some(tx),
                                            Err(err) => {
                                                println!("Unable to download logger file: {}", err);
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
                                        println!("Unable to download logger file: {}", err);
                                        None
                                    }
                                },
                            }
                        } else {
                            match path::download_file_to(&file_info.url, &log_path) {
                                Ok(_) => Some(tx),
                                Err(err) => {
                                    println!("Unable to download logger file: {}", err);
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
