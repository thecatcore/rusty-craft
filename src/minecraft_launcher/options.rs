use std::collections::hash_map::RandomState;
use std::collections::HashMap;

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
    pub height: Option<String>,
}

pub fn get_option_to_arg_map() -> HashMap<fn(LaunchOptions) -> String, &'static str, RandomState> {
    let mut map: HashMap<fn(LaunchOptions) -> String, &str> = HashMap::new();

    map.insert(
        |options: LaunchOptions| options.natives_directory,
        "natives_directory",
    );
    map.insert(
        |options: LaunchOptions| options.classpath,
        "classpath",
    );
    map.insert(
        |options: LaunchOptions| options.player_name,
        "auth_player_name",
    );
    map.insert(
        |options: LaunchOptions| options.version,
        "version_name",
    );
    map.insert(
        |options: LaunchOptions| options.game_directory,
        "game_directory",
    );
    map.insert(
        |options: LaunchOptions| options.assets_directory,
        "assets_root",
    );
    map.insert(
        |options: LaunchOptions| options.assets_index,
        "assets_index_name",
    );
    map.insert(
        |options: LaunchOptions| options.player_uuid,
        "auth_uuid",
    );
    map.insert(
        |options: LaunchOptions| options.player_token,
        "auth_access_token",
    );
    map.insert(
        |options: LaunchOptions| options.user_type,
        "user_type",
    );
    map.insert(
        |options: LaunchOptions| options.version_type,
        "version_type",
    );
    map.insert(
        |options: LaunchOptions| match options.width {
            None => String::from(""),
            Some(wid) => wid,
        },
        "resolution_width",
    );
    map.insert(
        |options: LaunchOptions| match options.height {
            None => String::from(""),
            Some(hei) => hei,
        },
        "resolution_height",
    );

    map
}
