use crate::editors::info::SystemArch;
use crate::live_api::release_info::UnityReleaseDownloadPlatform;
use directories::BaseDirs;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg_attr(target_os = "macos", path = "os/macos.rs")]
#[cfg_attr(target_os = "windows", path = "os/windows.rs")]
#[cfg_attr(target_os = "linux", path = "os/linux.rs")]
mod os;

pub fn get_platform() -> UnityReleaseDownloadPlatform {
    os::get_platform()
}

pub fn get_supported_editor_arch() -> Vec<SystemArch> {
    os::get_supported_editor_arch()
}

pub fn get_preferable_editor_arch() -> SystemArch {
    match std::env::consts::ARCH {
        "aarch64" => SystemArch::ARM64,
        _ => SystemArch::X86_64,
    }
}

pub fn get_config_path() -> PathBuf {
    let base_dirs = BaseDirs::new().unwrap();
    let path = PathBuf::from(base_dirs.config_dir()).join(os::get_config_folder_name());
    if let Err(err) = fs::create_dir_all(&path) {
        eprintln!("Warning: failed to create Unity Hub config directory {}: {}", path.display(), err);
    }
    path
}
pub fn get_default_install_path() -> PathBuf {
    let mut path = os::get_applications_path();
    path.push("Unity");
    path.push("Hub");
    path.push("Editor");
    if let Err(err) = fs::create_dir_all(&path) {
        eprintln!("Warning: failed to create default Unity install directory {}: {}", path.display(), err);
    }
    path
}

pub fn get_editor_install_move_path(editor_path: impl AsRef<Path>) -> Option<PathBuf> {
    os::get_editor_install_move_path(editor_path)
}

pub fn get_editor_executable_path(editor_path: impl AsRef<Path>) -> PathBuf {
    os::get_editor_executable_path(editor_path)
}

pub fn get_editor_executable_arch(editor_path: impl AsRef<Path>) -> Result<SystemArch, Box<dyn Error>> {
    os::get_editor_executable_arch(editor_path)
}
