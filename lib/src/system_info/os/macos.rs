use crate::editors::info::SystemArch;
use crate::live_api::release_info::UnityReleaseDownloadPlatform;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_platform() -> UnityReleaseDownloadPlatform {
    UnityReleaseDownloadPlatform::MAC_OS
}

pub fn get_supported_editor_arch() -> Vec<SystemArch> {
    match std::env::consts::ARCH {
        "aarch64" => vec![SystemArch::ARM64, SystemArch::X86_64],
        _ => vec![SystemArch::X86_64],
    }
}

pub fn get_config_folder_name() -> PathBuf {
    PathBuf::from("UnityHub")
}

pub fn get_applications_path() -> PathBuf {
    PathBuf::from("/Applications")
}

pub fn get_editor_install_move_path(editor_path: impl AsRef<Path>) -> Option<PathBuf> {
    let unpacked_path = PathBuf::from(editor_path.as_ref())
        .join("Unity");
    Some(unpacked_path)
}

pub fn get_editor_executable_path(editor_path: impl AsRef<Path>) -> PathBuf {
    let mut executable_path = PathBuf::from(editor_path.as_ref());
    executable_path.push("Unity.app");
    executable_path.push("Contents");
    executable_path.push("MacOS");
    executable_path.push("Unity");
    executable_path
}

pub fn get_editor_executable_arch(editor_path: impl AsRef<Path>) -> Result<SystemArch, Box<dyn Error>> {
    let file_info_output = Command::new("file").arg("--b").arg(editor_path.as_ref()).output()?.stdout;
    let file_info = String::from_utf8(file_info_output.clone())?;
    let file_info = file_info.trim().split(" ").last();
    match file_info {
        Some("arm64") => Ok(SystemArch::ARM64),
        _ => Ok(SystemArch::X86_64),
    }
}
