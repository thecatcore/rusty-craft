use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::manifest::version::{Either, Os, RuleAction};
use crate::minecraft_launcher::options::LaunchOptions;
use os_info::{get as get_os_info, Version};
use std::env::consts;
use std::ops::Add;

// pub fn get_args_from_manifest(version: &version::Main, options: &LaunchOptions) -> Option<String> {
//     match version.clone().arguments {
//         None => None,
//         Some(arguments) => match locate_java_home() {
//             Ok(path) => {
//                 let mut command = path.clone();
//
//                 match arguments.jvm {
//                     None => return None,
//                     Some(jvm_args) => {
//                         for i in jvm_args {
//                             match i {
//                                 Either::Left(string) => {
//                                     command = command
//                                         .add(String::from(" ").add(string.as_str()).as_str());
//                                 }
//                                 Either::Right(custom_arg) => {
//                                     match match_rules(custom_arg.rules, Some(options)) {
//                                         RuleAction::Allow => match custom_arg.value {
//                                             Either::Left(strin) => {
//                                                 command = command.add(
//                                                     String::from(" ").add(strin.as_str()).as_str(),
//                                                 );
//                                             }
//                                             Either::Right(strins) => {
//                                                 for i_str in strins {
//                                                     command = command.add(
//                                                         String::from(" ")
//                                                             .add(i_str.as_str())
//                                                             .as_str(),
//                                                     );
//                                                 }
//                                             }
//                                         },
//                                         RuleAction::Disallow => {}
//                                     }
//                                 }
//                             };
//                         }
//                     }
//                 };
//
//                 for i in arguments.game {
//                     match i {
//                         Either::Left(string) => {
//                             command = command.add(String::from(" ").add(string.as_str()).as_str());
//                         }
//                         Either::Right(custom_arg) => {
//                             match match_rules(custom_arg.rules, Some(options)) {
//                                 RuleAction::Allow => match custom_arg.value {
//                                     Either::Left(strin) => {
//                                         command = command
//                                             .add(String::from(" ").add(strin.as_str()).as_str());
//                                     }
//                                     Either::Right(strins) => {
//                                         for i_str in strins {
//                                             command = command.add(
//                                                 String::from(" ").add(i_str.as_str()).as_str(),
//                                             );
//                                         }
//                                     }
//                                 },
//                                 RuleAction::Disallow => {}
//                             }
//                         }
//                     };
//                 }
//
//                 Some(command)
//             }
//             Err(err) => {
//                 println!("Unable to locate java! {}", err);
//                 None
//             }
//         },
//     }
// }

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
