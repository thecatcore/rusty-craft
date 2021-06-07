use crate::minecraft_launcher::manifest::{version, assets};
use std::sync::mpsc::Sender;
use crate::minecraft_launcher::app::download_tab::Message;
use crate::minecraft_launcher::path;
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};

pub fn install_assets_index(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();

    tx.send(Message::NewStep(5)).unwrap_or(());
    tx.send(Message::NewSubStep(
        "Checking asset index".to_string(),
        1,
        3,
    ))
        .unwrap_or(());
    match version_manifest.asset_index {
        None => {
            tx.send(Message::Error(
                "Version manifest doesn't contain any asset index!".to_string(),
            ))
                .unwrap_or(());
            None
        }
        Some(a_index) => {
            // println!("Got asset index");
            match path::get_assets_folder(&String::from("indexes")) {
                None => {
                    tx.send(Message::Error("Unable to get indexes folder".to_string()))
                        .unwrap_or(());
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
                                            tx.send(Message::Error(format!(
                                                "Unable to download asset index file: {}",
                                                err_msg
                                            )))
                                                .unwrap_or(());
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
                                        tx.send(Message::Error(format!(
                                            "Unable to download asset index file: {}",
                                            err_msg
                                        )))
                                            .unwrap_or(());
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
                                tx.send(Message::Error(format!(
                                    "Unable to download asset index file: {}",
                                    err_msg
                                )))
                                    .unwrap_or(());
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
        "Installing missing assets".to_string(),
        2,
        3,
    ))
        .unwrap_or(());
    match path::get_assets_folder(&String::from("indexes")) {
        None => {
            tx.send(Message::Error("Unable to get indexes folder".to_string()))
                .unwrap_or(());
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
                                                tx.send(Message::Error(
                                                    "Unable to get objects folder".to_string(),
                                                ))
                                                    .unwrap_or(());
                                                None
                                            }
                                            Some(object_path) => {
                                                // println!("Got objects folder");
                                                let mut ret = Some(());
                                                let entry_count = main.objects.len();
                                                let mut entry_index = 0;
                                                for entry in main.objects.clone() {
                                                    entry_index += 1;

                                                    tx.send(Message::NewSubSubStep(
                                                        entry.0.clone().to_string(),
                                                        entry_index,
                                                        entry_count as u64,
                                                    ))
                                                        .unwrap_or(());

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
                                                                            tx.send(Message::Error(format!("Unable to download file {}: {}", entry.0, err)))
                                                                                .unwrap_or(());
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
                                                                        tx.send(Message::Error("Unable to create folder for asset".to_string()))
                                                                            .unwrap_or(());
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
                                                                                tx.send(Message::Error(format!("Unable to download file {}: {}", &entry.0, err)))
                                                                                    .unwrap_or(());
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
                                                                tx.send(Message::Error("Unable to create folder for asset".to_string()))
                                                                    .unwrap_or(());
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
                                                                        tx.send(Message::Error(format!("Unable to download file {}: {}", &entry.0, err)))
                                                                            .unwrap_or(());
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
                                                if main.map_to_resources {
                                                    tx.send(Message::NewSubStep(
                                                        "Relocating to resources folder"
                                                            .to_string(),
                                                        3,
                                                        3,
                                                    ))
                                                        .unwrap_or(());
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
                                                                    for (entry, asset_info) in
                                                                    main.objects
                                                                    {
                                                                        entry_index += 1;
                                                                        tx.send(
                                                                            Message::NewSubSubStep(
                                                                                entry
                                                                                    .clone()
                                                                                    .to_string(),
                                                                                entry_index,
                                                                                entry_count as u64,
                                                                            ),
                                                                        )
                                                                            .unwrap_or(());
                                                                        let hashed_path =
                                                                            asset_info
                                                                                .get_download_path(
                                                                                    &objects_path,
                                                                                );
                                                                        if hashed_path.1.exists() {
                                                                            match File::open(
                                                                                hashed_path.1,
                                                                            ) {
                                                                                Ok(mut file) => {
                                                                                    let mut body: Vec<u8> = Vec::new();
                                                                                    match file.read_to_end(&mut body) {
                                                                                        Ok(_) => {
                                                                                            if entry.contains('/') {
                                                                                                // let parts: Vec<&str> = entry.split("/").collect();
                                                                                                // let mut parts2: Vec<String> = Vec::new();
                                                                                                // for part in parts {
                                                                                                //     parts2.push(part.to_string());
                                                                                                // }
                                                                                                let entry_pathbuf = PathBuf::from(entry);

                                                                                                match path::get_or_create_dir(&resources, String::from(entry_pathbuf.parent().unwrap().to_str().unwrap())) {
                                                                                                    None => {
                                                                                                        tx.send(Message::Error("Unable to get asset path".to_string()))
                                                                                                            .unwrap_or(());
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
                                                                                                                        tx.send(Message::Error(format!("Unable to write to asset file: {}", err)))
                                                                                                                            .unwrap_or(());
                                                                                                                        res = None;
                                                                                                                        break;
                                                                                                                    }
                                                                                                                }
                                                                                                            }
                                                                                                            Err(err) => {
                                                                                                                tx.send(Message::Error(format!("Unable to create asset file: {}", err)))
                                                                                                                    .unwrap_or(());
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
                                                                                                                tx.send(Message::Error(format!("Unable to write to asset file: {}", err)))
                                                                                                                    .unwrap_or(());
                                                                                                                res = None;
                                                                                                                break;
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                    Err(err) => {
                                                                                                        tx.send(Message::Error(format!("Unable to create asset file: {}", err)))
                                                                                                            .unwrap_or(());
                                                                                                        res = None;
                                                                                                        break;
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        Err(err) => {
                                                                                            tx.send(Message::Error(format!("Unable to read asset file: {}", err)))
                                                                                                .unwrap_or(());
                                                                                            res = None;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Err(err) => {
                                                                                    tx.send(Message::Error(format!("Unable to open asset file: {}", err)))
                                                                                        .unwrap_or(());
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
                                    Err(err) => {
                                        tx.send(Message::Error(format!(
                                            "Unable to parse index file: {}",
                                            err
                                        )))
                                            .unwrap_or(());
                                        None
                                    }
                                }
                            }
                            Err(err) => {
                                tx.send(Message::Error(format!(
                                    "Unable to read index file: {}",
                                    err
                                )))
                                    .unwrap_or(());
                                None
                            }
                        }
                    }
                    Err(err) => {
                        tx.send(Message::Error(format!(
                            "Unable to opened index file: {}",
                            err
                        )))
                            .unwrap_or(());
                        None
                    }
                }
            } else {
                tx.send(Message::Error("Asset index file doesn't exist".to_string()))
                    .unwrap_or(());
                None
            }
        }
    }
}
