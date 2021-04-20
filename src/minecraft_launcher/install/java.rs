#[cfg(windows)]
use std::os::windows::fs::{symlink_dir, symlink_file};

#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::minecraft_launcher::app::download_tab::Message;
use crate::minecraft_launcher::manifest;
use crate::minecraft_launcher::manifest::java_versions::Version;
use crate::minecraft_launcher::manifest::{java_versions, version};
use crate::minecraft_launcher::path;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn check_java_version(
    version_manifest: &version::Main,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let version_manifest = version_manifest.clone();
    tx.send(Message::NewStep(2))
        .expect("Can't send message to renderer thread");
    tx.send(Message::NewSubStep(
        String::from("Downloading java versions manifest"),
        1,
        5,
    ))
    .expect("Can't send message to renderer thread");
    match get_java_version_manifest() {
        None => {
            tx.send(Message::NewSubStep(
                String::from("Checking if required version is installed"),
                3,
                5,
            ))
            .expect("Can't send message to renderer thread");
            match get_java_folder_path_sub(&version_manifest) {
                None => {
                    println!("Can't get java_folder_path_sub");
                    None
                }
                Some(java_folder) => {
                    match path::get_or_create_dirs(&java_folder, get_java_folder_for_os()) {
                        None => None,
                        Some(bin) => {
                            if (&java_folder).exists() {
                                if bin.join(get_java_ex_for_os()).exists() {
                                    tx.send(Message::NewSubStep(String::from("Done"), 5, 5))
                                        .expect("Can't send message to renderer thread");
                                    Some(tx)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
            }
        }

        Some(manifest) => match manifest.get_os_version() {
            None => {
                println!("Unable to get os_version");
                None
            }
            Some(os_version) => {
                tx.send(Message::NewSubStep(
                    String::from("Getting right java version"),
                    2,
                    5,
                ))
                .expect("Can't send message to renderer thread");
                let java_v_type = match version_manifest.java_version {
                    None => String::from("jre-legacy"),
                    Some(ver) => ver.component,
                };
                match os_version.get_java_version(&java_v_type) {
                    None => {
                        println!("Unable to get java_version");
                        None
                    }
                    Some(versions) => match versions.get(0) {
                        None => {
                            println!("Unable to get first version");
                            None
                        }
                        Some(version) => {
                            let online_version = version.clone().version.name;
                            tx.send(Message::NewSubStep(
                                String::from("Checking if required version is installed"),
                                3,
                                5,
                            ))
                            .expect("Can't send message to renderer thread");
                            match path::get_java_folder_path_sub(&java_v_type) {
                                None => {
                                    println!("Unable to get java_folder_path_sub");
                                    None
                                }
                                Some(j_folder) => match path::get_java_folder_path(&java_v_type) {
                                    None => {
                                        println!("Unable to get java_folder_path");
                                        None
                                    }
                                    Some(os_fol) => check_if_install_is_needed(
                                        j_folder,
                                        os_fol,
                                        java_v_type,
                                        version,
                                        online_version,
                                        tx,
                                    ),
                                },
                            }
                        }
                    },
                }
            }
        },
    }
}

fn check_if_install_is_needed(
    j_folder: PathBuf,
    os_fol: PathBuf,
    java_v_type: String,
    version: &Version,
    online_version: String,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    if (&j_folder).exists() {
        match File::open(os_fol.join(".version")) {
            Ok(mut v_file) => {
                let mut v_content = String::new();
                match v_file.read_to_string(&mut v_content) {
                    Ok(_) => {
                        if online_version != v_content {
                            install(
                                &java_v_type,
                                os_fol,
                                version.clone().manifest,
                                online_version,
                                tx,
                            )
                        } else {
                            // install(&java_v_type, os_fol, version.clone().manifest, online_version, tx)
                            Some(tx)
                        }
                    }
                    Err(_) => install(
                        &java_v_type,
                        os_fol,
                        version.clone().manifest,
                        online_version,
                        tx,
                    ),
                }
            }
            Err(_) => install(
                &java_v_type,
                os_fol,
                version.clone().manifest,
                online_version,
                tx,
            ),
        }
    } else {
        install(
            &java_v_type,
            os_fol,
            version.clone().manifest,
            online_version,
            tx,
        )
    }
}

fn get_java_folder_path_sub(version_manifest: &version::Main) -> Option<PathBuf> {
    path::get_java_folder_path_sub(
        &(match version_manifest.java_version.clone() {
            None => {
                // println!("Using default java version");
                String::from("jre-legacy")
            }
            Some(java_v) => {
                // println!("Found java version {}", java_v.component);
                java_v.component
            }
        }),
    )
}

fn install(
    java_v_type: &String,
    os_fol: PathBuf,
    manifest: java_versions::Manifest,
    online_version: String,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    tx.send(Message::NewSubStep(
        String::from("Installing missing files"),
        4,
        5,
    ))
    .expect("Can't send message to renderer thread");
    match install_java_version(&java_v_type, os_fol, manifest, online_version, tx) {
        None => None,
        Some(tx) => {
            tx.send(Message::NewSubStep(String::from("Done"), 5, 5))
                .expect("Can't send message to renderer thread");
            Some(tx)
        }
    }
}

fn install_java_version(
    type_: &String,
    os_folder: PathBuf,
    manifest: java_versions::Manifest,
    online_version: String,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let v_folder = match path::get_or_create_dir(&os_folder, type_.clone()) {
        None => {
            // println!("Failed to get v_folder");
            os_folder.clone()
        }
        Some(v) => {
            // println!("Got v_folder");
            v
        }
    };
    match path::read_file_from_url_to_string(&manifest.url) {
        Ok(stri) => {
            // println!("Read java_version_manifest");
            match manifest::java::parse_java_version_manifest(&stri) {
                Ok(manifest) => {
                    // println!("Parsed java_version_manifest");
                    let mut status: Option<()> = Some(());
                    let file_amount = manifest.files.len();
                    let mut current_file_index = 0;
                    for file in manifest.files {
                        if status.is_none() {
                            break;
                        }
                        current_file_index += 1;
                        let file_path = file.0;
                        tx.send(Message::NewSubSubStep(
                            format!("{}", file_path),
                            current_file_index,
                            (file_amount as u64) + 1,
                        ))
                        .expect("Can't send message to renderer thread");
                        let element_info = file.1;
                        let el_type = element_info.element_type;
                        let executable = match element_info.executable {
                            None => false,
                            Some(bool) => bool,
                        };
                        if el_type == "directory" {
                            if file_path.contains("/") {
                                let parts: Vec<&str> = file_path.split("/").collect();
                                let mut parts2: Vec<String> = Vec::new();
                                for part in parts {
                                    parts2.push(part.to_string());
                                }
                                let parts = parts2;
                                status = match path::get_or_create_dirs(&v_folder, parts) {
                                    None => None,
                                    Some(_) => Some(()),
                                }
                            } else {
                                status = match path::get_or_create_dir(&v_folder, file_path) {
                                    None => None,
                                    Some(_) => Some(()),
                                }
                            }
                        } else if el_type == "file" {
                            status = match element_info.downloads {
                                None => {
                                    println!("Failed to get download for file {}", file_path);
                                    None
                                }
                                Some(downloads) => {
                                    // println!("Got download for file {}", file_path);
                                    let url = downloads.raw.url;
                                    if file_path.contains("/") {
                                        // println!("File path contains '/'");
                                        let parts: Vec<&str> = file_path.split("/").collect();
                                        let mut parts2: Vec<String> = Vec::new();
                                        for part in parts {
                                            parts2.push(part.to_string());
                                        }
                                        match parts2.split_last() {
                                            None => {
                                                println!("Unable to split_last {}", file_path);
                                                None
                                            }
                                            Some(tuple) => {
                                                // println!("Split_lasted {}", file_path);
                                                let parts = Vec::from(tuple.1);
                                                match path::get_or_create_dirs(&v_folder, parts) {
                                                    None => {
                                                        println!("Unable to create folders");
                                                        None
                                                    }
                                                    Some(sub_pathh) => {
                                                        // println!("Created folders");
                                                        let file_buf = sub_pathh.join(tuple.0);
                                                        match path::download_file_to(
                                                            &url, &file_buf,
                                                        ) {
                                                            Ok(_) => {
                                                                // println!(
                                                                //     "Successfully downloaded file!"
                                                                // );
                                                                if executable {
                                                                    // println!("Executable");
                                                                    set_executable(file_buf)
                                                                } else {
                                                                    Some(())
                                                                }
                                                            }
                                                            Err(err) => {
                                                                println!(
                                                                    "Failed to download file: {}",
                                                                    err
                                                                );
                                                                None
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // println!("File path doesn't contain '/'");
                                        let file_buf = v_folder.join(file_path);
                                        match path::download_file_to(&url, &file_buf) {
                                            Ok(_) => {
                                                // println!("Successfully downloaded file");
                                                if executable {
                                                    // println!("Executable");
                                                    set_executable(file_buf)
                                                } else {
                                                    Some(())
                                                }
                                            }
                                            Err(err) => {
                                                println!("Failed to download file: {}", err);
                                                None
                                            }
                                        }
                                    }
                                }
                            };
                        } else if el_type == "link" {
                            status = create_symlink(&v_folder, file_path, element_info.target);
                        } else {
                            println!("Unknown el_type {}", el_type);
                        }
                    }
                    if status.is_some() {
                        tx.send(Message::NewSubSubStep(
                            format!(".version"),
                            (file_amount as u64) + 1,
                            (file_amount as u64) + 1,
                        ))
                        .expect("Can't send message to renderer thread");
                        let v_path = os_folder.join(".version");
                        match File::open(&v_path) {
                            Ok(mut v_path) => match v_path.write(online_version.as_bytes()) {
                                Ok(_) => {
                                    // println!("Wrote to .version file")
                                }
                                Err(_) => {
                                    println!("Failed to write to .version file");
                                    status = None
                                }
                            },
                            Err(_) => match File::create(v_path) {
                                Ok(mut v_path) => match v_path.write(online_version.as_bytes()) {
                                    Ok(_) => {
                                        // println!("Wrote to .version file")
                                    }
                                    Err(_) => {
                                        println!("Failed to write to .version file");
                                        status = None
                                    }
                                },
                                Err(err) => {
                                    println!("Failed to create .version file: {}", err);
                                    status = None;
                                }
                            },
                        }
                    }
                    match status {
                        None => None,
                        Some(_) => Some(tx),
                    }
                }
                Err(err) => {
                    println!("Failed to parse java_version_manifest {}", err);
                    None
                }
            }
        }
        Err(err) => {
            println!("Failed to read java_version_manifest {}", err);
            None
        }
    }
}

fn get_java_folder_for_os() -> Vec<String> {
    match std::env::consts::OS {
        "macos" => vec![
            String::from("jre.bundle"),
            String::from("Contents"),
            String::from("Home"),
            String::from("bin"),
        ],
        &_ => vec![String::from("bin")],
    }
}

fn get_java_ex_for_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "java.exe",
        &_ => "java",
    }
}

fn get_java_version_manifest() -> Option<java_versions::Main> {
    match path::read_file_from_url_to_string(&"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json".to_string()) {
        Ok(body) => {
            match java_versions::parse_java_versions_manifest(&body) {
                Ok(manifest) => Some(manifest),
                Err(err) => {
                    print!("Error: {}", err.to_string());
                    None
                }
            }
        }
        Err(err) => {
            print!("Error: {}", err);
            None
        }
    }
}

#[cfg(unix)]
fn set_executable(file_buf: PathBuf) -> Option<()> {
    match &file_buf.metadata() {
        Ok(meta) => {
            let mut perm = meta.permissions();
            perm.set_mode(0o755);
            match std::fs::set_permissions(file_buf, perm) {
                Ok(_) => Some(()),
                Err(err) => {
                    println!("Unable to set permission: {}", err);
                    None
                }
            }
        }
        Err(err) => {
            println!("Unable to get meta: {}", err);
            None
        }
    }
}

#[cfg(windows)]
fn set_executable(file_buf: PathBuf) -> Option<()> {
    Some(())
}

#[cfg(unix)]
fn create_symlink(v_folder: &PathBuf, path_name: String, target: Option<String>) -> Option<()> {
    match target {
        None => None,
        Some(target) => {
            let path_parts: Vec<&str> = path_name.split("/").collect();
            let target_parts: Vec<&str> = target.split("/").collect();

            let mut path_buf = v_folder.clone();
            for path_part in path_parts {
                path_buf = path_buf.join(path_part);
            }

            let mut target_buf = path_buf.clone();
            for path_part in target_parts {
                if path_part == ".." {
                    target_buf = match target_buf.parent() {
                        None => target_buf,
                        Some(p) => match p.to_path_buf().parent() {
                            None => p.to_path_buf(),
                            Some(p2) => p2.to_path_buf(),
                        },
                    };
                } else {
                    target_buf = target_buf.join(path_part);
                }
            }

            match symlink(target_buf, path_buf) {
                Ok(_) => Some(()),
                Err(_) => None,
            }
        }
    }
}

#[cfg(windows)]
fn create_symlink(v_folder: &PathBuf, path_name: String, target: Option<String>) -> Option<()> {
    println!("Symlink aren't handled on windows!");
    None
}