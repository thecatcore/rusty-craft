use serde_derive::Deserialize;
use std::collections::HashMap;
use std::ops::Add;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Main {
    pub objects: HashMap<String, AssetIndex>,
}

#[derive(Deserialize, Clone)]
pub struct AssetIndex {
    pub hash: String,
    pub size: u64,
}

impl AssetIndex {
    pub fn get_download_url(&self) -> String {
        let hach = &self.hash;
        let mut int: u8 = 0;
        let mut small = String::new();

        for hach_chr in hach.chars() {
            if int > 1 {
                break;
            }
            small = small.add(hach_chr.to_string().as_str());
            int += 1;
        }

        format!(
            "https://resources.download.minecraft.net/{}/{}",
            small, hach
        )
    }

    pub fn get_download_path(&self, object_path: &PathBuf) -> (String, PathBuf) {
        let hach = &self.hash;
        let mut int: u8 = 0;
        let mut small = String::new();

        for hach_chr in hach.chars() {
            if int > 1 {
                break;
            }
            small = small.add(hach_chr.to_string().as_str());
            int += 1;
        }

        (small.clone(), object_path.join(small).join(hach))
    }
}

pub fn parse(index: &String) -> serde_json::Result<Main> {
    let index_main: serde_json::Result<Main> = serde_json::from_str(index);

    index_main
}
