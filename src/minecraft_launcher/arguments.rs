use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::manifest::version::{Either, Os, RuleAction, Logging, ClientLogging, Rule};
use crate::minecraft_launcher::options::LaunchOptions;
use crate::minecraft_launcher::path;
use os_info::{get as get_os_info, Version};
use std::env::consts;
use std::ops::Add;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn get_args_from_manifest(version: &version::Main, options: &LaunchOptions) -> Option<Vec<String>> {
    match version.clone().arguments {
        None => None,
        Some(arguments) => {
            let mut command: Vec<String> = Vec::new();

            match arguments.jvm {
                None => {
                    command.push("-Djava.library.path=${natives_directory}".to_string());
                    command.push("-cp".to_string());
                    command.push("${classpath}".to_string());
                },
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
                        command.push(client_logging.argument);
                    }
                }
            }

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

    let mut version = version.clone();

    let separator = match get_os() {
        Os::Windows => ";",
        _ => ":"
    };

    for library in version.libraries {
        let mut allowed= match library.rules {
            None => RuleAction::Allow,
            Some(rules) => match_rules(rules, None)
        };

        match allowed {
            RuleAction::Allow => {
                let name_parts: Vec<&str> = library.name.split(":").collect();

                let lib_path = *name_parts.get(0).unwrap_or(&"");
                let name = *name_parts.get(1).unwrap_or(&"");
                let version = *name_parts.get(2).unwrap_or(&"");

                let file_name = match library.natives {
                    None => format!("{}-{}", name, version),
                    Some(natives) => {
                        match natives.get(get_os().to_str().as_str()) {
                            None => format!("{}-{}", name, version),
                            Some(native) => format!("{}-{}-{}", name, version, native),
                        }
                    }
                };

                match path::get_library_path(&format!("{}/{}/{}/{}.jar", lib_path, name, version, file_name)) {
                    None => {}
                    Some(lib_path) => {
                        native_arg = native_arg.add(format!("{}{}", lib_path.display(), separator).as_str());
                    }
                }
            }
            RuleAction::Disallow => {}
        }
    }

    match path::get_version_folder(&version.id) {
        None => {}
        Some(v_folder) => {
            native_arg = native_arg.add(format!("{}", v_folder.join(format!("{}.jar", version.id)).display()).as_str());
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
                        Some(opt) => opt.demo == true,
                    },
                    "has_custom_resolution" => match options {
                        None => false,
                        Some(opt) => opt.custom_resolution == true,
                    },
                    _ => false,
                };
                if !mat {
                    break;
                }
            }
            if mat {
                val = rule.action;
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
                            if v.starts_with("^") {
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
        "macos" => Os::MacOS,
        &_ => Os::Linux,
    }
}
