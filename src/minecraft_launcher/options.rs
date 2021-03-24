use std::collections::HashMap;
use std::collections::hash_map::RandomState;

pub struct LaunchOptions {
    pub natives_directory: String,
    pub classpath: String,
    pub player_name: String,
    pub version: String,
    pub game_directory: String,
    pub assets_directory: String,
    pub assets_index: String,
    pub player_uuid: String,
    pub player_token: String,
    pub user_type: String,
    pub version_type: String,
    pub demo: bool,
    pub custom_resolution: bool,
    pub width: Option<String>,
    pub height: Option<String>
}

pub fn get_option_to_arg_map() -> HashMap<fn(LaunchOptions) -> String, String, RandomState> {
    let mut map: HashMap<fn(LaunchOptions) -> String, String> = HashMap::new();

    map.insert(|options: LaunchOptions| options.natives_directory, String::from("natives_directory"));
    map.insert(|options: LaunchOptions| options.classpath, String::from("classpath"));
    map.insert(|options: LaunchOptions| options.player_name, String::from("auth_player_name"));
    map.insert(|options: LaunchOptions| options.version, String::from("version_name"));
    map.insert(|options: LaunchOptions| options.game_directory, String::from("game_directory"));
    map.insert(|options: LaunchOptions| options.assets_directory, String::from("assets_root"));
    map.insert(|options: LaunchOptions| options.assets_index, String::from("assets_index_name"));
    map.insert(|options: LaunchOptions| options.player_uuid, String::from("auth_uuid"));
    map.insert(|options: LaunchOptions| options.player_token, String::from("auth_access_token"));
    map.insert(|options: LaunchOptions| options.user_type, String::from("user_type"));
    map.insert(|options: LaunchOptions| options.version_type, String::from("version_type"));
    map.insert(|options: LaunchOptions|
                   match options.width {
                       None => { String::from("") }
                       Some(wid) => { wid }
                   }, String::from("resolution_width"));
    map.insert(|options: LaunchOptions|
                   match options.height {
                       None => { String::from("") }
                       Some(hei) => { hei }
                   }, String::from("resolution_height"));

    map
}