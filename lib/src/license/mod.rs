use std::error::Error;
use crate::system_info;
use fs4::fs_std::FileExt;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn lock_license(license_username: &str) -> Result<File, Box<dyn Error>> {
    let license_path = get_license_path();
    let common_lock_file = get_common_lock_file(&license_path)?;
    common_lock_file.lock_exclusive()?;
    let process_lock_path = license_path.join(format!("{}-{}.lock", license_username, uuid::Uuid::new_v4()));
    File::create(&process_lock_path)?;
    let process_lock_file = File::open(&process_lock_path)?;
    process_lock_file.lock_exclusive()?;
    common_lock_file.unlock()?;
    println!("{}", process_lock_path.display());
    Ok(process_lock_file)
}

pub fn release_license(license_username: &str, process_lock_file: File) -> Result<bool, Box<dyn Error>>{
    let license_path = get_license_path();
    let common_lock_file = get_common_lock_file(&license_path)?;
    common_lock_file.lock_exclusive()?;
    process_lock_file.unlock()?;

    let root = fs::read_dir(license_path);
    let mut license_locked = false;
    for child in root? {
        let entry = child?.path();
        if !entry.is_file() || entry.file_name().is_none() {
            continue;
        }
        let file_name = entry.file_name().unwrap().to_str().unwrap();
        if !file_name.starts_with(license_username){
            continue;
        }
        let process_lock_file = File::open(&entry)?;
        if process_lock_file.try_lock_exclusive().is_err() {
            license_locked = true;
        } else {
            fs::remove_file(&entry)?;
        }
    }

    common_lock_file.unlock()?;
    Ok(!license_locked)
}

fn get_license_path() -> PathBuf {
    let mut license_path = system_info::get_config_path();
    license_path.push("license");
    license_path
}

fn get_common_lock_file(license_path: impl AsRef<Path>) -> Result<File, Box<dyn Error>>{
    fs::create_dir_all(&license_path)?;
    let mut common_lock_path = PathBuf::from(license_path.as_ref());
    common_lock_path.push("common.lock");
    if !common_lock_path.exists() {
        File::create(&common_lock_path)?;
    }
    Ok(File::open(&common_lock_path)?)
}