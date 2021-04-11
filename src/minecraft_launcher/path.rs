use directories::BaseDirs;
use reqwest::blocking::{get as get_url, Response};
use std::env::consts;
use std::fs;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::PathBuf;

pub fn get_or_create_dir(current_folder: &PathBuf, sub: String) -> Option<PathBuf> {
    match current_folder.exists() {
        true => {
            let sub_path = current_folder.join(sub);

            match sub_path.exists() {
                true => Some(sub_path),
                false => match fs::create_dir(&sub_path) {
                    Ok(_) => Some(sub_path),
                    Err(e) => {
                        print!(
                            "Unable to create directory {} in folder {}: {}",
                            sub_path.file_name().expect("Ohno").to_str().expect("REE"),
                            current_folder
                                .file_name()
                                .expect("Ohno")
                                .to_str()
                                .expect("REE"),
                            e.to_string()
                        );
                        None
                    }
                },
            }
        }
        false => match fs::create_dir(current_folder) {
            Ok(_) => {
                let sub_path = current_folder.join(sub);

                match sub_path.exists() {
                    true => Some(sub_path),
                    false => match fs::create_dir(&sub_path) {
                        Ok(_) => Some(sub_path),
                        Err(e) => {
                            print!(
                                "Unable to create directory {} in folder {}: {}",
                                sub_path.file_name().expect("Ohno").to_str().expect("REE"),
                                current_folder
                                    .file_name()
                                    .expect("Ohno")
                                    .to_str()
                                    .expect("REE"),
                                e.to_string()
                            );
                            None
                        }
                    },
                }
            }
            Err(e) => {
                print!(
                    "Unable to create folder {}: {}",
                    current_folder
                        .file_name()
                        .expect("Ohno")
                        .to_str()
                        .expect("REE"),
                    e.to_string()
                );
                None
            }
        },
    }
}

pub fn get_or_create_dirs(current_folder: &PathBuf, sub: Vec<String>) -> Option<PathBuf> {
    let mut cur = current_folder.clone();
    for su in sub {
        match get_or_create_dir(&cur, su) {
            None => return None,
            Some(s) => cur = s.clone()
        };
    }
    Some(cur)
}

pub fn download_file_to(url: &String, path: &PathBuf) -> Result<String, String> {
    match read_file_from_url_to_type(url, AskedType::U8Vec) {
        Ok(u8Vec) => {
            match u8Vec {
                ReturnType::U8Vec(body) => {
                    let mut file = if path.exists() {
                        match File::open(path) {
                            Ok(file) => file,
                            Err(err) => {
                                return Err(format!("Failed to download {}: {}", url, err));
                            }
                        }
                    } else {
                        match File::create(path) {
                            Ok(file) => file,
                            Err(err) => {
                                return Err(format!("Failed to download {}: {}", url, err));
                            }
                        }
                    };

                    match file.write(&body) {
                        Ok(_) => Ok(format!(
                            "Successfully wrote {} to {}",
                            url,
                            path.file_name().expect("Ohno").to_str().expect("OhnoV2")
                        )),
                        Err(err) => Err(format!("Failed to download {}: {}", url, err)),
                    }
                }
                ReturnType::String(_) => {
                    Err(format!("Wrong Return type, expected Vec<u8> found String!"))
                }
            }
        }
        Err(err) => {
            Err(format!("Failed to download {}: {}", url, match err {
                ErrorType::STD(e) => { e.to_string() }
                ErrorType::Reqwest(e) => { e.to_string() }
            }))
        }
    }
}

pub fn read_file_from_url_to_string(url: &String) -> Result<String, String> {
    match read_file_from_url_to_type(url, AskedType::String) {
        Ok(string) => match string {
            ReturnType::U8Vec(_) => Err(format!("Wrong Return type, expected String found Vec<u8>!")),
            ReturnType::String(string) => Ok(string)
        }
        Err(err) => Err(format!("Failed to download {}: {}", url, match err {
            ErrorType::STD(e) => {e.to_string()}
            ErrorType::Reqwest(e) => {e.to_string()}
        }))
    }
}

pub fn read_file_from_url_to_type(url: &String, type_: AskedType) -> Result<ReturnType, ErrorType> {
    match get_url(url) {
        Ok(mut data) => {
            match type_ {
                AskedType::U8Vec => {
                    let mut body: Vec<u8> = Vec::new();
                    match data.read_to_end(&mut body) {
                        Ok(_) => Ok(ReturnType::U8Vec(body)),
                        Err(err) => Err(ErrorType::STD(err))
                    }
                }
                AskedType::String => {
                    let mut body = String::new();
                    match data.read_to_string(&mut body) {
                        Ok(_) => Ok(ReturnType::String(body)),
                        Err(err) => Err(ErrorType::STD(err))
                    }
                }
            }
        }
        Err(err) => {
            Err(ErrorType::Reqwest(err))
        }
    }
}

pub enum ReturnType {
    U8Vec(Vec<u8>),
    String(String)
}

pub enum ErrorType {
    STD(Error),
    Reqwest(reqwest::Error)
}

pub enum AskedType {
    U8Vec,
    String
}

pub fn get_version_folder(version: &String) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("versions")) {
        None => None,
        Some(vs) => get_or_create_dir(&vs, version.clone()),
    }
}

pub fn get_assets_folder(sub: &String) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("assets")) {
        None => None,
        Some(vs) => get_or_create_dir(&vs, sub.clone()),
    }
}

pub fn get_library_path(sub: &String) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("assets")) {
        None => None,
        Some(vs) => {
            if sub.contains("/") {
                let mut subs: Vec<&str> = sub.split("/").collect();

                let file_name = subs.remove(subs.len() - 1);

                let mut path = vs;

                for sub in subs {
                    let sub_path = get_or_create_dir(&path, String::from(sub));
                    match sub_path {
                        None => return None,
                        Some(s_path) => {
                            path = s_path;
                        }
                    }
                }

                Some(path.join(file_name))
            } else {
                get_or_create_dir(&vs, sub.clone())
            }
        }
    }
}

pub fn get_java_folder_path(type_: &String) -> Option<PathBuf> {
    match get_minecraft_sub_folder(&String::from("runtime")) {
        None => None,
        Some(runtime) => match get_or_create_dir(&runtime, type_.clone()) {
            None => None,
            Some(type1) => match get_or_create_dir(&type1, String::from(get_os_java_name())) {
                None => None,
                Some(os) => Some(os)
            }
        }
    }
}

pub fn get_java_folder_path_sub(type_: &String) -> Option<PathBuf> {
    match get_java_folder_path(type_) {
        None => None,
        Some(os) => Some(os.join(type_))
    }
}

fn get_os_java_name() -> &'static str {
    match consts::OS {
        "windows" => match consts::ARCH {
            "x86" => "windows-x86",
            "x86_64" => "windows-x64",
            &_ => ""
        },
        "macos" => "mac-os",
        &_ => match consts::ARCH {
            "x86" => "linux-i386",
            &_ => "linux"
        }
    }
}

pub fn get_minecraft_sub_folder(sub: &String) -> Option<PathBuf> {
    get_or_create_dir(&get_minecraft_directory(), sub.to_string())
}

fn get_minecraft_directory_name() -> &'static str {
    match consts::OS {
        "macos" => "minecraft",
        &_ => ".minecraft",
    }
}

pub fn get_minecraft_directory() -> PathBuf {
    let base_dir: BaseDirs = BaseDirs::new().expect("Can't get base directories!");

    let dir = match consts::OS {
        "windows" => base_dir.data_dir(),
        "macos" => base_dir.data_dir(),
        &_ => base_dir.home_dir(),
    };

    let min_dir = dir.join(get_minecraft_directory_name());

    min_dir
}
