use crate::live_api;
use crate::live_api::release_info::{DownloadSize, ModuleDownload, PathRename, ReleaseDigitalUnit};
use crate::modules::info::SizeUnitType::Value;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SizeUnitType {
    UnitSize(DownloadSize),
    Value(f64),
}

impl From<SizeUnitType> for f64 {
    fn from(value: SizeUnitType) -> Self {
        match value {
            SizeUnitType::UnitSize(unit_size) => unit_size.into(),
            Value(value) => value,
        }
    }
}

impl From<DownloadSize> for f64 {
    fn from(value: DownloadSize) -> Self {
        match value.unit {
            ReleaseDigitalUnit::BYTE => value.value as f64,
            ReleaseDigitalUnit::KILOBYTE => value.value as f64 * 1024.0,
            ReleaseDigitalUnit::MEGABYTE => value.value as f64 * 1024.0 * 1024.0,
            ReleaseDigitalUnit::GIGABYTE => value.value as f64 * 1024.0 * 1024.0 * 1024.0,
            _ => unimplemented!(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModuleInfo {
    pub url: String,
    pub integrity: Option<String>,
    #[serde(rename = "type")]
    pub module_type: live_api::release_info::FileType,
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub category: String,
    #[serde(rename = "downloadSize")]
    pub download_size: SizeUnitType,
    #[serde(rename = "installedSize")]
    pub installed_size: SizeUnitType,
    pub required: bool,
    pub hidden: bool,
    #[serde(rename = "extractedPathRename")]
    pub extracted_path_rename: Option<PathRename>,
    #[serde(rename = "preSelected")]
    pub pre_selected: bool,
    pub destination: Option<String>,
    #[serde(rename = "subModules")]
    pub submodules: Option<Vec<ModuleDownload>>,
    pub __typename: String,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    pub visible: bool,
    pub selected: bool,
    pub sync: String,
    pub parent: String,
    #[serde(rename = "eulaUrl1")]
    pub eula_url_1: String,
    #[serde(rename = "eulaLabel1")]
    pub eula_label_1: String,
    #[serde(rename = "eulaMessage")]
    pub eula_message: String,
    #[serde(rename = "renameTo")]
    pub rename_to: String,
    #[serde(rename = "renameFrom")]
    pub rename_from: String,
    pub preselected: bool,
}

impl From<ModuleDownload> for ModuleInfo {
    fn from(module_download: ModuleDownload) -> Self {
        let mut info = ModuleInfo {
            url: module_download.url.clone(),
            integrity: module_download.integrity,
            module_type: module_download.type_,
            id: module_download.id,
            name: module_download.name,
            slug: module_download.slug,
            description: module_download.description,
            category: module_download.category.into(),
            download_size: Value(f64::from(module_download.download_size.download_size)),
            installed_size: Value(f64::from(module_download.installed_size.download_size)),
            required: module_download.required,
            hidden: module_download.hidden,
            extracted_path_rename: None,
            pre_selected: module_download.pre_selected,
            destination: module_download.destination,
            submodules: None,
            __typename: String::from("UnityReleaseModule"),
            download_url: module_download.url,
            visible: !module_download.hidden,
            selected: false,
            sync: String::new(),
            parent: String::new(),
            eula_url_1: String::new(),
            eula_label_1: String::new(),
            eula_message: String::new(),
            rename_to: String::new(),
            rename_from: String::new(),
            preselected: module_download.pre_selected,
        };
        if let Some(extracted_rename) = module_download.extracted_path_rename {
            info.rename_from = extracted_rename.path_rename.from.clone();
            info.rename_to = extracted_rename.path_rename.to.clone();
            info.extracted_path_rename = Some(extracted_rename.path_rename);
        }
        if let Some(eula) = module_download.eula {
            if eula.len() > 0 {
                let eula = eula.first().unwrap();
                info.eula_url_1 = eula.url.clone();
                info.eula_label_1 = eula.label.clone();
                info.eula_message = eula.message.clone();
            }
        }
        info
    }
}

impl PartialEq for ModuleInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ModuleInfo {}

impl Hash for ModuleInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

pub fn convert_api_modules(release_download: &live_api::release_info::ReleaseDownload) -> Result<Vec<ModuleInfo>, Box<dyn Error>> {
    let mut result_list: Vec<ModuleInfo> = Vec::new();
    for root_module in &release_download.modules {
        let mut root_module_info = ModuleInfo::from(root_module.module_download.clone());
        root_module_info.visible = true;
        if let Some(submodules1) = &root_module.sub_modules {
            let mut root_module_children: Vec<ModuleDownload> = Vec::new();
            for submodule1 in submodules1 {
                root_module_children.push(submodule1.module_download.clone());
                let mut submodule1_info = ModuleInfo::from(submodule1.module_download.clone());
                let mut submodules1_download_size = submodule1_info.download_size.clone().into();
                let mut submodules1_install_size = submodule1_info.installed_size.clone().into();
                if let Some(submodules2) = &submodule1.sub_modules {
                    let mut submodule1_children: Vec<ModuleDownload> = Vec::new();
                    for submodule2 in submodules2 {
                        submodule1_children.push(submodule2.module_download.clone());
                        let mut submodule2_info = ModuleInfo::from(submodule2.module_download.clone());
                        if submodule2_info.required && !submodule2_info.visible {
                            submodule2_info.sync = submodule1_info.id.clone();
                            submodules1_download_size += f64::from(submodule2_info.download_size.clone());
                            submodules1_install_size += f64::from(submodule2_info.installed_size.clone());
                        }
                        submodule2_info.parent = submodule1_info.id.clone();
                        result_list.push(submodule2_info);
                    }
                    submodule1_info.download_size = Value(submodules1_download_size);
                    submodule1_info.installed_size = Value(submodules1_install_size);
                    submodule1_info.submodules = Some(submodule1_children);
                }
                if submodule1_info.required && !submodule1_info.visible {
                    submodule1_info.sync = root_module_info.id.clone();
                }
                submodule1_info.parent = root_module_info.id.clone();
                result_list.push(submodule1_info);
            }
            root_module_info.submodules = Some(root_module_children);
        }
        result_list.push(root_module_info);
    }
    Ok(result_list)
}
