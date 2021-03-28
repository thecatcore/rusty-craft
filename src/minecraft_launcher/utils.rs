use reqwest::blocking::get as get_url;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn get_body_from_url_else_from_file(url: &str, path: &PathBuf) -> Option<String> {
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
