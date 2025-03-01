use crate::editors::info::SystemArch;
use crate::live_api::release_info::UnityReleaseDownloadPlatform;
use directories::BaseDirs;
use std::error::Error;
use std::path::{Path, PathBuf};

pub fn get_platform() -> UnityReleaseDownloadPlatform {
    UnityReleaseDownloadPlatform::LINUX
}

pub fn get_supported_editor_arch() -> Vec<SystemArch> {
    vec![SystemArch::X86_64]
}

pub fn get_config_folder_name() -> PathBuf {
    PathBuf::from("unityhub")
}

pub fn get_applications_path() -> PathBuf {
    let base_dirs = BaseDirs::new().unwrap();
    base_dirs.home_dir().to_path_buf()
}

pub fn get_editor_install_move_path(_editor_path: impl AsRef<Path>) -> Option<PathBuf> {
    None
}

pub fn get_editor_executable_path(editor_path: impl AsRef<Path>) -> PathBuf {
    let mut executable_path = PathBuf::from(editor_path.as_ref());
    executable_path.push("Editor");
    executable_path.push("Unity");
    executable_path
}

pub fn get_editor_executable_arch(_editor_path: impl AsRef<Path>) -> Result<SystemArch, Box<dyn Error>> {
    Ok(SystemArch::X86_64)
}
