use crate::editors;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
fn apply_bee_workaround(version: &str, arch: Option<String>) -> Result<(), Box<dyn Error>> {
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn apply_bee_workaround(version: &str, arch: Option<String>) -> Result<(), Box<dyn Error>> {
    let editor_path = editors::get_installed_editor_path(version, arch)?;
    if let Some(editor_path) = editor_path {
        for entry in WalkDir::new(editor_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let filename = entry.file_name().to_string_lossy();
            if filename != "bee_backend" || !entry.file_type().is_file() {
                continue;
            }
            let mut real_bee_path = entry_path.as_os_str().to_owned();
            real_bee_path.push("_real");
            let real_bee_path = PathBuf::from(real_bee_path);
            if real_bee_path.exists() {
                continue;
            }
            println!("Applying bee workaround to '{}'", entry_path.display());
            fs::rename(entry_path, &real_bee_path)?;
            fs::write(entry_path, "\
        #! /bin/bash
        args=(\"$@\")
        for ((i=0; i<\"${#args[@]}\"; ++i))
        do
            case ${args[i]} in
                --stdin-canary)
                    unset args[i];
                    break;;
            esac
        done
        ${0}_real \"${args[@]}\"
        ")?;
            let perms = fs::metadata(real_bee_path)?.permissions();
            fs::set_permissions(entry_path, perms)?;
        }
        return Ok(());
    }
    Err("Editor not found".into())
}