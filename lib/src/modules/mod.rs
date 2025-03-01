pub mod info;

use crate::modules::info::ModuleInfo;
use crate::{editors, install, system_info};
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
    let mut modules = HashMap::new();
    let editor_executable_path = system_info::get_editor_executable_path(path.as_ref());
    if !editor_executable_path.exists() {
        return Err("No editor found".into());
    }
    let modules_path = PathBuf::from(path.as_ref()).join("modules.json");
    match modules_path.exists() {
        true => {
            let contents = fs::read_to_string(modules_path)?;
            let info: Vec<ModuleInfo> = serde_json::from_str(&contents)?;
            for s in info {
                modules.insert(s.id.clone(), s);
            }
            Ok(modules)
        }
        false => Err("modules.json not found".into()),
    }
}

pub fn write_modules_info(path: impl AsRef<Path>, modules: Vec<ModuleInfo>) -> Result<(), Box<dyn Error>> {
    let modules_path = PathBuf::from(path.as_ref()).join("modules.json");
    fs::write(modules_path, serde_json::to_string(&modules)?)?;
    Ok(())
}
