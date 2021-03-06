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
use std::path::{Path, PathBuf};
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
        Err(_err) => {
            tx.send(Message::NewSubStep(
                String::from("Checking if required version is installed"),
                3,
                5,
            ))
            .unwrap_or(());
            match get_java_folder_path_sub(&version_manifest) {
                None => {
                    tx.send(Message::Error(String::from(
                        "Can't get java_folder_path_sub",
                    )))
                    .unwrap_or(());
                    None
                }
                Some(java_folder) => {
                    match path::get_or_create_dir(&java_folder, get_java_folder_for_os()) {
                        None => {
                            tx.send(Message::Error(format!(
                                "Unable to get or create java folder: {}",
                                java_folder.display()
                            )))
                            .unwrap_or(());
                            None
                        }
                        Some(bin) => {
                            if (&java_folder).exists() {
                                if bin.join(get_java_ex_for_os()).exists() {
                                    tx.send(Message::NewSubStep(String::from("Done"), 5, 5))
                                        .unwrap_or(());
                                    Some(tx)
                                } else {
                                    tx.send(Message::Error(format!(
                                        "Unable to find java executable: {}",
                                        bin.join(get_java_ex_for_os()).display()
                                    )))
                                    .unwrap_or(());
                                    None
                                }
                            } else {
                                tx.send(Message::Error(format!(
                                    "Unable to find java folder: {}",
                                    java_folder.display()
                                )))
                                .unwrap_or(());
                                None
                            }
                        }
                    }
                }
            }
        }

        Ok(manifest) => match manifest.get_os_version() {
            None => {
                tx.send(Message::Error(String::from("Unable to get os_version")))
                    .unwrap_or(());
                None
            }
            Some(os_version) => {
                tx.send(Message::NewSubStep(
                    String::from("Getting right java version"),
                    2,
                    5,
                ))
                .unwrap_or(());
                let java_v_type = match version_manifest.java_version {
                    None => String::from("jre-legacy"),
                    Some(ver) => ver.component,
                };
                match os_version.get_java_version(&java_v_type) {
                    None => {
                        tx.send(Message::Error(format!(
                            "Unable to get java_version from type '{}'",
                            java_v_type
                        )))
                        .unwrap_or(());
                        None
                    }
                    Some(versions) => match versions.get(0) {
                        None => {
                            tx.send(Message::Error(String::from(
                                "Unable to get first java version",
                            )))
                            .unwrap_or(());
                            None
                        }
                        Some(version) => {
                            let online_version = version.clone().version.name;
                            tx.send(Message::NewSubStep(
                                String::from("Checking if required version is installed"),
                                3,
                                5,
                            ))
                            .unwrap_or(());
                            match path::get_java_folder_path_sub(&java_v_type) {
                                None => {
                                    tx.send(Message::Error(format!(
                                        "Unable to get java_folder_path_sub from type '{}'",
                                        java_v_type
                                    )))
                                    .unwrap_or(());
                                    None
                                }
                                Some(j_folder) => match path::get_java_folder_path(&java_v_type) {
                                    None => {
                                        tx.send(Message::Error(format!(
                                            "Unable to get java_folder_path from type '{}'",
                                            java_v_type
                                        )))
                                        .unwrap_or(());
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
            None => String::from("jre-legacy"),
            Some(java_v) => java_v.component,
        }),
    )
}

fn install(
    java_v_type: &str,
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
    .unwrap_or(());
    match install_java_version(&java_v_type, os_fol, manifest, online_version, tx) {
        None => None,
        Some(tx) => {
            tx.send(Message::NewSubStep(String::from("Done"), 5, 5))
                .unwrap_or(());
            Some(tx)
        }
    }
}

fn install_java_version(
    type_: &str,
    os_folder: PathBuf,
    manifest: java_versions::Manifest,
    online_version: String,
    tx: Sender<Message>,
) -> Option<Sender<Message>> {
    let v_folder = match path::get_or_create_dir(&os_folder, type_.to_string()) {
        None => os_folder.clone(),
        Some(v) => v,
    };
    match path::read_file_from_url_to_string(&manifest.url) {
        Ok(stri) => {
            match manifest::java::parse_java_version_manifest(&stri) {
                Ok(manifest) => {
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
                            file_path.to_string(),
                            current_file_index,
                            (file_amount as u64) + 1,
                        ))
                        .unwrap_or(());
                        let element_info = file.1;
                        let el_type = element_info.element_type;
                        let executable = element_info.executable;
                        if el_type == "directory" {
                            status = match path::get_or_create_dir(&v_folder, file_path.clone()) {
                                None => {
                                    tx.send(Message::Error(format!(
                                        "Unable to create folder {} in folder {}",
                                        file_path,
                                        &v_folder.display()
                                    )))
                                    .unwrap_or(());
                                    None
                                }
                                Some(_) => Some(()),
                            }
                        } else if el_type == "file" {
                            status = match element_info.downloads {
                                None => {
                                    tx.send(Message::Error(format!(
                                        "Failed to get download for file {}",
                                        file_path
                                    )))
                                    .unwrap_or(());
                                    None
                                }
                                Some(downloads) => {
                                    let url = downloads.raw.url;
                                    if file_path.contains('/') {
                                        let file_pathbuf = PathBuf::from(file_path);
                                        match path::get_or_create_dir(
                                            &v_folder,
                                            String::from(
                                                file_pathbuf.parent().unwrap().to_str().unwrap(),
                                            ),
                                        ) {
                                            None => {
                                                tx.send(Message::Error(
                                                    "Unable to create folders".to_string(),
                                                ))
                                                .unwrap_or(());
                                                None
                                            }
                                            Some(sub_pathh) => {
                                                // println!("Created folders");
                                                let file_buf = sub_pathh.join(
                                                    file_pathbuf.components().last().unwrap(),
                                                );
                                                match path::download_file_to(&url, &file_buf) {
                                                    Ok(_) => {
                                                        // println!(
                                                        //     "Successfully downloaded file!"
                                                        // );
                                                        if executable {
                                                            // println!("Executable");
                                                            match set_executable(file_buf) {
                                                                Ok(_) => Some(()),
                                                                Err(err) => {
                                                                    tx.send(Message::Error(err))
                                                                        .unwrap_or(());
                                                                    None
                                                                }
                                                            }
                                                        } else {
                                                            Some(())
                                                        }
                                                    }
                                                    Err(err) => {
                                                        tx.send(Message::Error(format!(
                                                            "Failed to download file: {}",
                                                            err
                                                        )))
                                                        .unwrap_or(());
                                                        None
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
                                                    match set_executable(file_buf) {
                                                        Ok(_) => Some(()),
                                                        Err(err) => {
                                                            tx.send(Message::Error(err))
                                                                .unwrap_or(());
                                                            None
                                                        }
                                                    }
                                                } else {
                                                    Some(())
                                                }
                                            }
                                            Err(err) => {
                                                tx.send(Message::Error(format!(
                                                    "Failed to download file: \n{}",
                                                    err
                                                )))
                                                .unwrap_or(());
                                                None
                                            }
                                        }
                                    }
                                }
                            };
                        } else if el_type == "link" {
                            status = create_symlink(
                                &v_folder,
                                file_path,
                                element_info.target,
                                tx.clone(),
                            );
                        } else {
                            tx.send(Message::Error(format!("Unknown el_type {}", el_type)))
                                .unwrap_or(());
                        }
                    }
                    if status.is_some() {
                        tx.send(Message::NewSubSubStep(
                            ".version".to_string(),
                            (file_amount as u64) + 1,
                            (file_amount as u64) + 1,
                        ))
                        .unwrap_or(());
                        let v_path = os_folder.join(".version");
                        match File::open(&v_path) {
                            Ok(mut v_path) => match v_path.write(online_version.as_bytes()) {
                                Ok(_) => {
                                    // println!("Wrote to .version file")
                                }
                                Err(_) => {
                                    tx.send(Message::Error(
                                        "Failed to write to .version file".to_string(),
                                    ))
                                    .unwrap_or(());
                                    status = None
                                }
                            },
                            Err(_) => match File::create(v_path) {
                                Ok(mut v_path) => match v_path.write(online_version.as_bytes()) {
                                    Ok(_) => {
                                        // println!("Wrote to .version file")
                                    }
                                    Err(_) => {
                                        tx.send(Message::Error(
                                            "Failed to write to .version file".to_string(),
                                        ))
                                        .unwrap_or(());
                                        status = None
                                    }
                                },
                                Err(err) => {
                                    tx.send(Message::Error(format!(
                                        "Failed to create .version file: {}",
                                        err
                                    )))
                                    .unwrap_or(());
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
                    tx.send(Message::Error(format!(
                        "Failed to parse java_version_manifest {}",
                        err
                    )))
                    .unwrap_or(());
                    None
                }
            }
        }
        Err(err) => {
            tx.send(Message::Error(format!(
                "Failed to read java_version_manifest {}",
                err
            )))
            .unwrap_or(());
            None
        }
    }
}

pub fn get_java_folder_for_os() -> String {
    match std::env::consts::OS {
        "macos" => String::from("jre.bundle/Contents/Home/bin"),
        &_ => String::from("bin"),
    }
}

pub fn get_java_ex_for_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "java.exe",
        &_ => "java",
    }
}

fn get_java_version_manifest() -> Result<java_versions::Main, String> {
    match path::read_file_from_url_to_string(&"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json".to_string()) {
        Ok(body) => {
            match java_versions::parse_java_versions_manifest(&body) {
                Ok(manifest) => Ok(manifest),
                Err(err) => {
                    Err(format!("Error: {}", err.to_string()))
                }
            }
        }
        Err(err) => {
            Err(format!("Error: {}", err))
        }
    }
}

#[cfg(unix)]
fn set_executable(file_buf: PathBuf) -> Result<(), String> {
    match &file_buf.metadata() {
        Ok(meta) => {
            let mut perm = meta.permissions();
            perm.set_mode(0o755);
            match std::fs::set_permissions(file_buf, perm) {
                Ok(_) => Ok(()),
                Err(err) => Err(format!("Unable to set permission: {}", err)),
            }
        }
        Err(err) => Err(format!("Unable to get meta: {}", err)),
    }
}

#[cfg(windows)]
fn set_executable(file_buf: PathBuf) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn create_symlink(
    v_folder: &Path,
    path_name: String,
    target: Option<String>,
    tx: Sender<Message>,
) -> Option<()> {
    match target {
        None => {
            tx.send(Message::Error("Link target is none!".to_string()))
                .unwrap_or(());
            None
        }
        Some(target) => {
            let path_buffer = PathBuf::from(path_name.clone());

            match path_buffer.parent() {
                None => {}
                Some(p) => {
                    match path::get_or_create_dir(&v_folder, p.display().to_string()) {
                        None => {
                            tx.send(Message::Error(
                                "Failed to create folder in which symlink is!".to_string(),
                            ))
                            .unwrap_or(());
                            return None;
                        }
                        Some(_) => {}
                    };
                }
            }

            let path_parts: Vec<&str> = path_name.split('/').collect();
            let target_parts: Vec<&str> = target.split('/').collect();

            let mut path_buf = PathBuf::from(v_folder);
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
                Err(err) => {
                    tx.send(Message::Error(format!("Failed to create symlink: {}", err)))
                        .unwrap_or(());
                    None
                }
            }
        }
    }
}

#[cfg(windows)]
fn create_symlink(
    v_folder: &PathBuf,
    path_name: String,
    target: Option<String>,
    tx: Sender<Message>,
) -> Option<()> {
    tx.send(Message::Error(format!(
        "Symlink aren't handled on windows!"
    )))
    .unwrap_or(());
    None
}
