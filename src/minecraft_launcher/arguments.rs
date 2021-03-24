use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::manifest::version::{Either, RuleAction, Os};
use crate::minecraft_launcher::options::LaunchOptions;
use java_locator::locate_java_home;
use std::ops::Add;
use std::env::consts;
use os_info::{get as get_os_info, Version};

pub fn get_args_from_manifest(version: &version::Main, options: &LaunchOptions) -> Option<String> {
    match version.clone().arguments {
        None => {None}
        Some(arguments) => {
            match locate_java_home() {
                Ok(path) => {
                    let mut command = path.clone();

                    match arguments.jvm {
                        None => {return None}
                        Some(jvm_args) => {

                            for i in jvm_args {
                                match i {
                                    Either::Left(string) => {
                                        command = command.add(String::from(" ").add(string.as_str()).as_str());
                                    }
                                    Either::Right(custom_arg) => {
                                        match match_rules(custom_arg.rules, options) {
                                            RuleAction::Allow => {
                                                match custom_arg.value {
                                                    Either::Left(strin) => {
                                                        command = command.add(String::from(" ").add(strin.as_str()).as_str());
                                                    }
                                                    Either::Right(strins) => {
                                                        for i_str in strins {
                                                            command = command.add(String::from(" ").add(i_str.as_str()).as_str());
                                                        }
                                                    }
                                                }
                                            }
                                            RuleAction::Disallow => {}
                                        }
                                    }
                                };
                            };

                        }
                    };

                    for i in arguments.game {
                        match i {
                            Either::Left(string) => {
                                command = command.add(String::from(" ").add(string.as_str()).as_str());
                            }
                            Either::Right(custom_arg) => {
                                match match_rules(custom_arg.rules, options) {
                                    RuleAction::Allow => {
                                        match custom_arg.value {
                                            Either::Left(strin) => {
                                                command = command.add(String::from(" ").add(strin.as_str()).as_str());
                                            }
                                            Either::Right(strins) => {
                                                for i_str in strins {
                                                    command = command.add(String::from(" ").add(i_str.as_str()).as_str());
                                                }
                                            }
                                        }
                                    }
                                    RuleAction::Disallow => {}
                                }
                            }
                        };
                    };

                    Some(command)
                }
                Err(err) => {
                    println!("Unable to locate java! {}", err);
                    None
                }
            }
        }
    }
}

fn match_rules(rules: Vec<version::ArgumentRule>, options: &LaunchOptions) -> RuleAction {
    let mut val: RuleAction = RuleAction::Allow;

    for rule in rules {
        if rule.features.is_some() {
            let mut mat = true;
            for i in rule.features.expect("Wut") {
                mat = match i.0.as_str() {
                    "is_demo_user" => options.demo == true,
                    "has_custom_resolution" => options.custom_resolution == true,
                    _ => false
                };
                if !mat { break }
            }
            if mat {
                val = rule.action;
            }
        } else if rule.os.is_some() {
            let mut mat = true;
            for i in rule.os.expect("Wut") {
                mat = match i.0.as_str() {
                    "name" => i.1 == get_os().to_str(),
                    "version" => match get_os_info().version() {
                        Version::Unknown => {false}
                        Version::Semantic(maj, _min, _pat) => {
                            let v = i.1.replace("^", "").replace("\\", "").replace(".", "");
                            let v_64: u64 = v.parse().unwrap();
                            maj >= &v_64
                        }
                        Version::Rolling(_) => {false}
                        Version::Custom(_) => {false}
                    },
                    "arch" => i.1 == consts::ARCH,
                    _ => false
                };
                if !mat { break }
            }
            if mat {
                val = rule.action;
            }
        } else {
            val = rule.action;
        };
    };

    val
}

fn get_os() -> Os {
    match consts::OS {
        "windows" => Os::Windows,
        "macos" => Os::MacOS,
        &_ => Os::Linux,
    }
}