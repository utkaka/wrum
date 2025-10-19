use crate::live_api::release_info::FileType;
use crate::system_info;
use serde_json::Value;
use std::error::Error;
use std::{fs};
use std::fs::File;
use std::path::{Path, PathBuf};
use fs4::fs_std::FileExt;

mod download;
mod unpack;

static CONFIG_FILE_NAME: &str = "secondaryInstallPath.json";

pub fn get_install_lock(id: &str, editor_path: &Path) -> Result<File, Box<dyn Error>> {
    fs::create_dir_all(editor_path)?;
    let mut lock_file_path = PathBuf::from(editor_path);
    lock_file_path.push(format!(".{}.lock", id));
    if !lock_file_path.exists() {
        File::create(&lock_file_path)?;
    }
    let lock_file = File::open(&lock_file_path)?;
    if let Err(_) = lock_file.try_lock_exclusive() {
        println!("Another process is already installing \"{}\". Waiting...", id);
    }
    lock_file.lock_exclusive()?;
    Ok(lock_file)
}

pub fn release_install_lock(lock_file: File) -> Result<(), Box<dyn Error>> {
    lock_file.unlock()?;
    Ok(())
}

pub fn install(url: &str, id: &str, editor_path: &Path, module_type: FileType, destination: &str,
               rename_from: &str, rename_to: &str) -> Result<(), Box<dyn Error>> {
    let download_path = download::download(url, id, editor_path)?;

    println!("Unpacking {}.", id);
    unpack::unpack(module_type.clone(), &download_path, get_in_editor_path(editor_path.to_str().unwrap(), destination))?;
    if !rename_from.is_empty() && !rename_to.is_empty() {
        let rename_from = get_in_editor_path(editor_path.to_str().unwrap(), rename_from);
        let rename_to = get_in_editor_path(editor_path.to_str().unwrap(), rename_to);
        unpack::move_files(rename_from, rename_to)?;
    }

    fs::remove_file(download_path)?;
    println!("{} successfully installed.", id);
    Ok(())
}

pub fn get_install_path() -> Result<PathBuf, Box<dyn Error>> {
    let secondary_path = get_secondary_install_path()?;
    match secondary_path {
        None => Ok(system_info::get_default_install_path()),
        Some(value) => Ok(value)
    }
}

pub fn set_secondary_install_path(path: impl AsRef<Path>, check_path: bool) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    if check_path && !path.exists() {
        return Err("Error: This directory does not exist.".into());
    }
    let config_file_path = system_info::get_config_path().join(CONFIG_FILE_NAME);
    let json = serde_json::to_string(&path)?;
    match fs::write(config_file_path, json) {
        Ok(_) => Ok(()),
        Err(_) => Err("Couldn't write secondaryInstallPath.json.".into()),
    }
}

pub fn get_secondary_install_path() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let config_file_path = system_info::get_config_path().join(CONFIG_FILE_NAME);
    if !config_file_path.exists() {
        let empty_value = serde_json::to_string("")?;
        fs::write(&config_file_path, empty_value)?;
        return Ok(None);
    }
    let contents = fs::read_to_string(config_file_path)?;
    match serde_json::from_str(&contents)? {
        Value::String(string) => match string.is_empty() {
            true => Ok(None),
            false => Ok(Some(PathBuf::from(string))),
        },
        _ => Err("Couldn't read secondaryInstallPath.json.".into()),
    }
}

fn get_in_editor_path(editor_path: &str, relative_path: &str) -> PathBuf {
    PathBuf::from(relative_path.replace("{UNITY_PATH}", editor_path))
}
