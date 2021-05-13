use serde_derive::Deserialize;
use std::collections::HashMap;
use std::ops::Add;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone)]
pub struct Main {
    pub objects: HashMap<String, AssetIndex>,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Deserialize, Clone)]
pub struct AssetIndex {
    pub hash: String,
    pub size: u64,
}

impl AssetIndex {
    pub fn get_download_url(&self) -> String {
        let hach = &self.hash;
        let mut small = String::new();

        for (int, hach_chr) in hach.chars().enumerate() {
            if int > 1 {
                break;
            }
            small = small.add(hach_chr.to_string().as_str());
        }

        format!(
            "https://resources.download.minecraft.net/{}/{}",
            small, hach
        )
    }

    pub fn get_download_path(&self, object_path: &Path) -> (String, PathBuf) {
        let hach = &self.hash;
        let mut small = String::new();

        for (int, hach_chr) in hach.chars().enumerate() {
            if int > 1 {
                break;
            }
            small = small.add(hach_chr.to_string().as_str());
        }

        (small.clone(), object_path.join(small).join(hach))
    }
}

pub fn parse(index: &str) -> serde_json::Result<Main> {
    let index_main: serde_json::Result<Main> = serde_json::from_str(index);

    index_main
}
