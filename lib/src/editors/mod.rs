pub mod info;

use crate::editors::info::{read_editor_info, write_editor_info, EditorInfo, SystemArch};
use crate::install::get_install_path;
use crate::live_api::release_info;
use crate::{install, live_api, modules, system_info};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::string::String;

pub fn install_editor(version: &str, architecture: Option<String>) -> Result<i32, Box<dyn Error>> {
    let preferable_arch = match architecture.clone() {
        None => system_info::get_preferable_editor_arch(),
        Some(arch_str) => SystemArch::from(arch_str),
    };

    let installed = list_installed_version(&version)?;
    if get_installed_editor_info(&installed, preferable_arch.clone()).is_some() {
        println!("{} already installed!", version);
        return Ok(0);
    }

    let info = live_api::get_version_info(&version, system_info::get_platform(), system_info::get_supported_editor_arch())?;
    if info.is_none() {
        return Err("Couldn't find release".into());
    }
    let info = info.unwrap();
    let mut download: Option<release_info::ReleaseDownload> = None;
    for release_download in info.downloads {
        let release_info::ReleaseDownloads::UnityReleaseHubDownload(release_download) = release_download;
        let download_arch = SystemArch::from(release_download.architecture.clone());
        if download.is_none() || download_arch == preferable_arch {
            download = Some(release_download);
        }
    }
    if download.is_none() {
        return Err("Couldn't find any download for release".into());
    }

    let download = download.unwrap();
    let download_arch = SystemArch::from(download.architecture.clone());

    if let Some(arch_str) = architecture {
        if SystemArch::from(arch_str) != download_arch {
            return Err("Couldn't find any download for specified arch".into());
        }
    }

    if get_installed_editor_info(&installed, download_arch.clone()).is_some() {
        println!("{} already installed!", version);
        return Ok(0);
    }

    let mut version_path = version.to_string();
    if !installed.is_empty() {
        version_path = format!("{} ({})", version_path, String::from(download_arch.clone()));
    }
    let mut editor_path = get_install_path()?;

    let mut required_disk_space: u64 = 0;
    let installed_size: f64 = download.installed_size.clone().download_size.into();
    let download_size: f64 = download.download_size.clone().download_size.into();
    required_disk_space += (installed_size + download_size) as u64;

    if fs4::available_space(editor_path.clone()).unwrap() < required_disk_space {
        return Err("Not enough free disk space".into());
    }
    editor_path.push(version_path);

    let lock_file = install::get_install_lock(&version, &editor_path)?;
    let installed = list_installed_version(&version)?;
    if get_installed_editor_info(&installed, download_arch.clone()).is_none() {
        install::install(&download.url, &version, &editor_path, download.type_.clone(), "{UNITY_PATH}",
                         system_info::get_editor_install_move_path(&editor_path).unwrap_or(PathBuf::new()).to_str().unwrap(),
                         &editor_path.to_str().unwrap())?;

        let modules = modules::info::convert_api_modules(&download);
        modules::write_modules_info(&editor_path, modules.unwrap())?;

        write_editor_info(
            editor_path,
            EditorInfo {
                version: version.to_string(),
                path: PathBuf::new(),
                executable_path: PathBuf::new(),
                arch: SystemArch::from(download.architecture),
            },
        )?;
    }

    install::release_install_lock(lock_file)?;

    Ok(0)
}

pub fn list_installed_version(version: &str) -> Result<Vec<EditorInfo>, Box<dyn Error>> {
    let mut installed = Vec::new();
    let editors = list_installed_editors()?;
    for editor in editors {
        if editor.version != version {
            continue;
        }
        installed.push(editor);
    }
    Ok(installed)
}

pub fn list_installed_editors() -> Result<Vec<EditorInfo>, Box<dyn Error>> {
    let mut editors = Vec::new();
    list_editors_in_folder(system_info::get_default_install_path(), &mut editors)?;
    append_secondary_path_editors(&mut editors)?;
    Ok(editors)
}

pub fn get_installed_editor_info(installed: & Vec<EditorInfo>, arch: SystemArch) -> Option<&EditorInfo> {
    for editor in installed {
        if editor.arch == arch {
            return Some(editor);
        }
    }
    None
}

pub fn get_installed_editor_path(version: &str, arch: Option<String>) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let installed_version = list_installed_version(version)?;
    match arch {
        Some(arch_str) => {
            match get_installed_editor_info(&installed_version, SystemArch::from(arch_str)) {
                None => Ok(None),
                Some(editor_info) => Ok(Some(editor_info.path.clone()))
            }
        },
        None => {
            let preferable_arch = system_info::get_preferable_editor_arch();
            let mut path = None;
            for installation in installed_version {
                if path.is_none() || installation.arch == preferable_arch {
                    path = Some(installation.path.clone());
                }
            }
            Ok(path)
        }
    }
}

fn append_secondary_path_editors(editors: &mut Vec<EditorInfo>) -> Result<(), Box<dyn Error>> {
    let secondary_path = install::get_secondary_install_path()?;
    if let Some(secondary_path) = secondary_path {
        list_editors_in_folder(secondary_path, editors)?;
    }
    Ok(())
}

fn list_editors_in_folder(path: PathBuf, editors: &mut Vec<EditorInfo>) -> Result<(), Box<dyn Error>> {
    let root = fs::read_dir(path);
    for child in root? {
        let entry = child?.path();
        if !entry.is_dir() {
            continue;
        }
        if let Some(item) = read_editor_info(entry)? {
            editors.push(item);
        }
    }
    Ok(())
}
