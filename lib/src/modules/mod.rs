pub mod info;

use crate::modules::info::ModuleInfo;
use crate::{editors, install, live_api, system_info};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn install_modules(version: &str, modules: Vec<String>, arch: Option<String>, include_children: bool) -> Result<(), Box<dyn Error>> {
    let editor_path = editors::get_installed_editor_path(version, arch)?;
    if let Some(editor_path) = editor_path {
        let mut modules_to_install = vec![];
        let mut editor_modules_info = read_modules_info(&editor_path)?;
        for module_in_args in modules {
            push_module_to_install(module_in_args, &editor_modules_info, &mut modules_to_install, include_children);
        }

        if modules_to_install.is_empty() {
            return Ok(());
        }

        let mut required_disk_space: u64 = 0;
        for module_to_install in modules_to_install.clone() {
            if module_to_install.sync.is_empty() {
                continue;
            }
            let installed_size: f64 = module_to_install.installed_size.into();
            let download_size: f64 = module_to_install.download_size.into();
            required_disk_space += (installed_size + download_size) as u64;
        }

        if fs4::available_space(editor_path.clone()).unwrap() < required_disk_space {
            return Err("Not enough free disk space".into());
        }

        for module_to_install in modules_to_install {
            let destination = module_to_install.destination.unwrap_or(String::from("/Applications"));

            let lock_file = install::get_install_lock(&module_to_install.id, &editor_path)?;
            editor_modules_info = read_modules_info(&editor_path)?;
            if !editor_modules_info[&module_to_install.id].selected {
                install::install(&module_to_install.url, &module_to_install.id, &editor_path,
                                 module_to_install.module_type.clone(), &destination,
                                 &module_to_install.rename_from, &module_to_install.rename_to)?;
                editor_modules_info.get_mut(&module_to_install.id).unwrap().selected = true;
                write_modules_info(&editor_path, editor_modules_info.values().cloned().collect())?;
            }
            install::release_install_lock(lock_file)?;
        }

        return Ok(());
    }
    Err("Editor not found".into())
}

fn push_module_to_install(module: String, modules_info: &HashMap<String, ModuleInfo>, modules_to_install: &mut Vec<ModuleInfo>, include_children: bool) {
    if !modules_info.contains_key(&module) {
        return;
    }
    for module_in_list in modules_to_install.clone() {
        if module == module_in_list.id {
            return;
        }
    }
    let module = modules_info[&module].clone();
    let parent_module = module.parent.clone();
    if !parent_module.is_empty() && !modules_info[&parent_module].selected {
        push_module_to_install(parent_module, modules_info, modules_to_install, false);
    }
    if module.selected != true {
        modules_to_install.push(module.clone());
    }

    if let Some(submodules) = module.submodules {
        for submodule in submodules {
            let submodule_info = modules_info[&submodule.id].clone();
            if include_children || submodule_info.sync == module.id {
                push_module_to_install(submodule.id, modules_info, modules_to_install, include_children);
            }
        }
    }
}

pub fn read_modules_info(path: impl AsRef<Path>) -> Result<HashMap<String, ModuleInfo>, Box<dyn Error>> {
    let editor_executable_path = system_info::get_editor_executable_path(path.as_ref());
    if !editor_executable_path.exists() {
        return Err("No editor found".into());
    }
    let modules_vec = match load_modules_from_disk(path.as_ref()) {
        Ok(modules) => modules,
        Err(err) => {
            eprintln!(
                "modules.json is missing or invalid for {} ({}). Attempting to refresh metadata...",
                path.as_ref().display(),
                err
            );
            rebuild_modules_metadata(path.as_ref())?
        }
    };
    Ok(modules_vec.into_iter().fold(HashMap::new(), |mut acc, module| {
        acc.insert(module.id.clone(), module);
        acc
    }))
}

pub fn write_modules_info(path: impl AsRef<Path>, modules: Vec<ModuleInfo>) -> Result<(), Box<dyn Error>> {
    let modules_path = PathBuf::from(path.as_ref()).join("modules.json");
    fs::write(modules_path, serde_json::to_string(&modules)?)?;
    Ok(())
}

fn load_modules_from_disk(path: &Path) -> Result<Vec<ModuleInfo>, Box<dyn Error>> {
    let modules_path = PathBuf::from(path).join("modules.json");
    let contents = fs::read_to_string(modules_path)?;
    let info: Vec<ModuleInfo> = serde_json::from_str(&contents)?;
    Ok(info)
}

fn rebuild_modules_metadata(path: &Path) -> Result<Vec<ModuleInfo>, Box<dyn Error>> {
    let editor_info = editors::info::read_editor_info(PathBuf::from(path))?
        .ok_or("Couldn't read editor metadata for installed editor")?;
    let release = live_api::get_version_info(
        &editor_info.version,
        system_info::get_platform(),
        vec![editor_info.arch.clone()],
    )?
    .ok_or("Couldn't retrieve release info for installed editor")?;
    let download = release
        .downloads
        .into_iter()
        .find_map(|release_download| {
            #[allow(irrefutable_let_patterns)]
            if let live_api::release_info::ReleaseDownloads::UnityReleaseHubDownload(download) = release_download {
                let download_arch = editors::info::SystemArch::from(download.architecture.clone());
                if download_arch == editor_info.arch {
                    return Some(download);
                }
            }
            None
        })
        .ok_or("Couldn't locate download for installed editor architecture")?;
    let modules = crate::modules::info::convert_api_modules(&download)?;
    write_modules_info(path, modules.clone())?;
    Ok(modules)
}
