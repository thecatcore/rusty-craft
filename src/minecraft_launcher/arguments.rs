use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::manifest::version::{Either, Logging, Os, Rule, RuleAction, AssetIndex};
use crate::minecraft_launcher::path;
use os_info::{get as get_os_info, Version};

use std::env::consts;
use std::ops::Add;
use std::path::PathBuf;

pub fn get_args_from_manifest(
    version: &version::Main,
    options: &LaunchOptions,
) -> Option<Vec<String>> {
    match version.clone().arguments {
        None => {
            match version.clone().minecraft_arguments {
                None => None,
                Some(minecraft_arguments) => {
                    let mut command: Vec<String> = Vec::new();

                    command.push("-Djava.library.path=${natives_directory}".to_string());
                    command.push("-cp".to_string());
                    command.push("${classpath}".to_string());

                    command.push(version.clone().main_class);

                    let arguments: Vec<&str> = minecraft_arguments.split(" ").collect();

                    for argument in arguments {
                        command.push(argument.to_string());
                    }

                    Some(command)
                }
            }
        },
        Some(arguments) => {
            let mut command: Vec<String> = Vec::new();

            match arguments.jvm {
                None => {
                    command.push("-Djava.library.path=${natives_directory}".to_string());
                    command.push("-cp".to_string());
                    command.push("${classpath}".to_string());
                }
                Some(jvm_args) => {
                    for i in jvm_args {
                        match i {
                            Either::Left(string) => {
                                command.push(string);
                            }
                            Either::Right(custom_arg) => {
                                match match_rules(custom_arg.rules, Some(options)) {
                                    RuleAction::Allow => match custom_arg.value {
                                        Either::Left(strin) => {
                                            command.push(strin);
                                        }
                                        Either::Right(strins) => {
                                            for i_str in strins {
                                                command.push(i_str);
                                            }
                                        }
                                    },
                                    RuleAction::Disallow => {}
                                }
                            }
                        };
                    }
                }
            };

            match version.clone().logging {
                None => {}
                Some(logging) => match logging.client {
                    None => {}
                    Some(client_logging) => {
                        match path::get_assets_folder("log_configs") {
                            None => {}
                            Some(log_config_folder) => {
                                let log_path = log_config_folder.join(client_logging.file.id).display().to_string();
                                command.push(client_logging.argument.replace("${path}", log_path.as_str()));
                            }
                        };
                    }
                },
            }

            command.push(version.clone().main_class);

            for i in arguments.game {
                match i {
                    Either::Left(string) => {
                        command.push(string);
                    }
                    Either::Right(custom_arg) => {
                        match match_rules(custom_arg.rules, Some(options)) {
                            RuleAction::Allow => match custom_arg.value {
                                Either::Left(strin) => {
                                    command.push(strin);
                                }
                                Either::Right(strins) => {
                                    for i_str in strins {
                                        command.push(i_str);
                                    }
                                }
                            },
                            RuleAction::Disallow => {}
                        }
                    }
                };
            }

            Some(command)
        }
    }
}

pub fn get_natives(version: &version::Main) -> String {
    let mut native_arg = String::new();

    let version = version.clone();

    let separator = match get_os() {
        Os::Windows => ";",
        _ => ":",
    };

    for library in version.libraries {
        let allowed = match library.rules {
            None => RuleAction::Allow,
            Some(rules) => match_rules(rules, None),
        };

        match allowed {
            RuleAction::Allow => {
                let name_parts: Vec<&str> = library.name.split(':').collect();

                let lib_path = *name_parts.get(0).unwrap_or(&"");
                let name = *name_parts.get(1).unwrap_or(&"");
                let version = *name_parts.get(2).unwrap_or(&"");

                let file_name = match library.natives {
                    None => format!("{}-{}", name, version),
                    Some(natives) => match natives.get(get_os().to_str().as_str()) {
                        None => format!("{}-{}", name, version),
                        Some(native) => format!("{}-{}-{}", name, version, native),
                    },
                };

                match path::get_library_path(&format!(
                    "{}/{}/{}/{}.jar",
                    lib_path, name, version, file_name
                )) {
                    None => {}
                    Some(lib_path) => {
                        native_arg =
                            native_arg.add(format!("{}{}", lib_path.display(), separator).as_str());
                    }
                }
            }
            RuleAction::Disallow => {}
        }
    }

    match path::get_version_folder(&version.id) {
        None => {}
        Some(v_folder) => {
            native_arg = native_arg.add(
                format!("{}", v_folder.join(format!("{}.jar", version.id)).display()).as_str(),
            );
        }
    }

    native_arg
}

pub fn match_rules(rules: Vec<version::Rule>, options: Option<&LaunchOptions>) -> RuleAction {
    let mut val: RuleAction = RuleAction::Allow;

    for rule in rules {
        if rule.features.is_some() {
            let mut mat = true;
            for i in rule.features.expect("Wut") {
                mat = match i.0.as_str() {
                    "is_demo_user" => match options {
                        None => false,
                        Some(opt) => opt.demo,
                    },
                    "has_custom_resolution" => match options {
                        None => false,
                        Some(opt) => opt.custom_resolution,
                    },
                    _ => false,
                };
                if !mat {
                    break;
                }
            }
            if mat {
                val = rule.action;
            } else {
                val = match rule.action {
                    RuleAction::Allow => RuleAction::Disallow,
                    RuleAction::Disallow => RuleAction::Allow
                }
            }
        } else if rule.os.is_some() {
            let mut mat = true;
            for i in rule.os.expect("Wut") {
                if !mat {
                    break;
                }
                mat = match i.0.as_str() {
                    "name" => i.1 == get_os().to_str(),
                    "version" => match get_os_info().version() {
                        Version::Unknown => false,
                        Version::Semantic(maj, min, pat) => {
                            let mut v = i.1.replace("\\", "").replace("d$", "0");

                            let mut sup_or_equ = false;
                            if v.starts_with('^') {
                                sup_or_equ = true;
                                v = v.replace("^", "");
                            }

                            match Version::from_string(v) {
                                Version::Unknown => false,
                                Version::Semantic(major, minor, patch) => {
                                    if sup_or_equ {
                                        major > *maj
                                            || major == *maj
                                                && (minor > *min || minor == *min && patch >= *pat)
                                    } else {
                                        *maj == major && *min == minor && *pat == patch
                                    }
                                }
                                Version::Rolling(_) => false,
                                Version::Custom(_) => false,
                            }
                        }
                        Version::Rolling(_) => false,
                        Version::Custom(_) => false,
                    },
                    "arch" => i.1 == consts::ARCH,
                    _ => false,
                };
            }
            if mat {
                val = rule.action;
            } else {
                val = match rule.action {
                    RuleAction::Allow => RuleAction::Disallow,
                    RuleAction::Disallow => RuleAction::Allow,
                }
            }
        } else {
            val = rule.action;
        };
    }

    val
}

pub fn get_os() -> Os {
    match consts::OS {
        "windows" => Os::Windows,
        "macos" => Os::MacOs,
        &_ => Os::Linux,
    }
}

#[derive(Clone)]
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

impl LaunchOptions {
    pub fn from_version(version: &version::Main, player_name: String, player_uuid: String, player_token: String, user_type: String) -> Result<LaunchOptions, &str> {
        let version = version.clone();

        let natives_directory = match path::get_bin_folder(version.id.clone()) {
            None => {
                return Err("Unable to get natives directory!");
            }
            Some(dir) => dir.display().to_string()
        };

        let classpath = get_natives(&version);

        let game_directory = path::get_minecraft_directory().display().to_string();

        let assets_directory = path::get_minecraft_directory().join("assets").display().to_string();

        let assets_index = match version.asset_index {
            None => match version.assets {
                None => {
                    return Err("Unable to get asset index");
                }
                Some(index) => index
            }
            Some(index) => index.id
        };

        let version_type = version._type.to_string();

        Ok(LaunchOptions {
            natives_directory,
            classpath,
            player_name,
            version: version.id,
            game_directory,
            assets_directory,
            assets_index,
            player_uuid,
            player_token,
            user_type,
            version_type,
            demo: false,
            custom_resolution: false,
            width: None,
            height: None
        })
    }

    pub fn fill_argument_list(&mut self, args: Vec<String>) -> Vec<String> {
        let mut new_args: Vec<String> = Vec::new();

        for arg in args {
            let mut arg = arg.replace("${natives_directory}", self.natives_directory.as_str());
            arg = arg.replace("${launcher_name}", "Rusty-Craft");
            arg = arg.replace("${launcher_version}", crate::get_version().as_str());
            arg = arg.replace("${classpath}", self.classpath.as_str());
            arg = arg.replace("${auth_player_name}", self.player_name.as_str());
            arg = arg.replace("${version_name}", self.version.as_str());
            arg = arg.replace("${game_directory}", self.game_directory.as_str());
            arg = arg.replace("${assets_root}", self.assets_directory.as_str());
            arg = arg.replace("${assets_index_name}", self.assets_index.as_str());
            arg = arg.replace("${auth_uuid}", self.player_uuid.as_str());
            arg = arg.replace("${auth_access_token}", self.player_token.as_str());
            arg = arg.replace("${user_type}", self.user_type.as_str());
            arg = arg.replace("${version_type}", self.version_type.as_str());
            arg = arg.replace("${user_properties}", "{}");
            match self.width.clone() {
                None => {
                    arg = arg.replace("${resolution_width}", "854");
                }
                Some(width) => {
                    arg = arg.replace("${resolution_width}", width.as_str());
                }
            }
            match self.height.clone() {
                None => {
                    arg = arg.replace("${resolution_height}", "480");
                }
                Some(height) => {
                    arg = arg.replace("${resolution_height}", height.as_str());
                }
            }
            arg = arg.replace("${game_assets}", path::get_minecraft_directory().join("resources").display().to_string().as_str());
            arg = arg.replace("${auth_session}", self.player_token.as_str());

            new_args.push(arg);
        }

        new_args
    }
}