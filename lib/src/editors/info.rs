use crate::system_info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum SystemArch {
    X86_64,
    ARM64,
}

impl From<String> for SystemArch {
    fn from(value: String) -> Self {
        match value.as_str() {
            "x86_64" => SystemArch::X86_64,
            "arm64" => SystemArch::ARM64,
            _ => SystemArch::X86_64,
        }
    }
}

impl From<SystemArch> for String {
    fn from(value: SystemArch) -> Self {
        match value {
            SystemArch::X86_64 => String::from("x86_64"),
            SystemArch::ARM64 => String::from("arm64")
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EditorInfo {
    pub version: String,
    #[serde(skip)]
    pub path: PathBuf,
    #[serde(skip)]
    pub executable_path: PathBuf,
    pub arch: SystemArch,
}

pub fn read_editor_info(path: PathBuf) -> Result<Option<EditorInfo>, Box<dyn Error>> {
    let editor_executable_path = system_info::get_editor_executable_path(path.clone());
    if !editor_executable_path.exists() {
        return Ok(None);
    }
    let cached_info_path = PathBuf::from(path.clone()).join("wrum.json");
    match cached_info_path.exists() {
        true => {
            let contents = fs::read_to_string(cached_info_path)?;
            let mut info: EditorInfo = serde_json::from_str(&contents)?;
            info.path = path;
            info.executable_path = editor_executable_path;
            Ok(Some(info))
        }
        false => {
            let editor_version_output = Command::new(editor_executable_path.clone())
                .arg("-batchmode")
                .arg("-nographics")
                .arg("--version")
                .output()?
                .stdout;
            let editor_version = String::from_utf8(editor_version_output)?.trim().to_string();
            let editor_item = EditorInfo {
                version: editor_version,
                arch: system_info::get_editor_executable_arch(editor_executable_path.clone())?,
                path,
                executable_path: editor_executable_path,
            };
            let json = serde_json::to_string(&editor_item)?;
            if let Err(_) = fs::write(cached_info_path, json) {
                return Err("Couldn't write wrum.json".into());
            }
            Ok(Some(editor_item))
        }
    }
}

pub fn write_editor_info(path: PathBuf, info: EditorInfo) -> Result<(), Box<dyn Error>> {
    let cached_info_path = PathBuf::from(path.clone()).join("wrum.json");
    let json = serde_json::to_string(&info)?;
    if let Err(_) = fs::write(cached_info_path, json) {
        return Err("Couldn't write wrum.json".into());
    }
    Ok(())
}
