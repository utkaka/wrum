use crate::editors::info::SystemArch;
use crate::modules::info::SizeUnitType;
use ::reqwest::blocking::Client;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use std::error::Error;

static API_URL: &str = "https://live-platform-api.prd.ld.unity3d.com/graphql";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/live_api/graphql/schema.graphql",
    query_path = "src/live_api/graphql/major_release_list.graphql",
    response_derives = "Debug,Serialize,Clone"
)]
pub struct LatestMajorReleases;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/live_api/graphql/schema.graphql",
    query_path = "src/live_api/graphql/release_info.graphql",
    response_derives = "Debug,Serialize,Clone"
)]
pub struct ReleaseInfo;

type DateTime = String;
type URL = String;
type SubresourceIntegrity = String;

impl From<release_info::UnityReleaseModuleCategory> for String {
    fn from(value: release_info::UnityReleaseModuleCategory) -> Self {
        match value {
            release_info::UnityReleaseModuleCategory::DOCUMENTATION => String::from("Documentation"),
            release_info::UnityReleaseModuleCategory::PLATFORM => String::from("Platforms"),
            release_info::UnityReleaseModuleCategory::LANGUAGE_PACK => String::from("Language packs (Preview)"),
            release_info::UnityReleaseModuleCategory::DEV_TOOL => String::from("Dev tools"),
            release_info::UnityReleaseModuleCategory::PLUGIN => String::from("Plugins"),
            release_info::UnityReleaseModuleCategory::COMPONENT => String::from("Components"),
            release_info::UnityReleaseModuleCategory::Other(_) => String::new(),
        }
    }
}

impl From<release_info::DownloadSize> for SizeUnitType {
    fn from(value: release_info::DownloadSize) -> Self {
        SizeUnitType::UnitSize(value)
    }
}

impl From<SystemArch> for release_info::UnityReleaseDownloadArchitecture {
    fn from(arch: SystemArch) -> release_info::UnityReleaseDownloadArchitecture {
        match arch {
            SystemArch::X86_64 => release_info::UnityReleaseDownloadArchitecture::X86_64,
            SystemArch::ARM64 => release_info::UnityReleaseDownloadArchitecture::ARM64,
        }
    }
}

impl From<release_info::UnityReleaseDownloadArchitecture> for SystemArch {
    fn from(arch: release_info::UnityReleaseDownloadArchitecture) -> SystemArch {
        match arch {
            release_info::UnityReleaseDownloadArchitecture::X86_64 => SystemArch::X86_64,
            release_info::UnityReleaseDownloadArchitecture::ARM64 => SystemArch::ARM64,
            _ => SystemArch::X86_64,
        }
    }
}

pub fn get_major_release_list() -> Result<Vec<latest_major_releases::LatestMajorReleasesGetUnityReleaseMajorVersions>, Box<dyn Error>> {
    let variables = latest_major_releases::Variables {
        platform: Some(vec![latest_major_releases::UnityReleaseDownloadPlatform::MAC_OS]),
    };
    let client = Client::new();
    let response_body = post_graphql::<LatestMajorReleases, _>(&client, API_URL, variables)?;
    if let Some(response) = response_body.data {
        return Ok(response.get_unity_release_major_versions);
    }
    Err("Couldn't retrieve major release list".into())
}

pub fn get_version_info(
    version: &str,
    platform: release_info::UnityReleaseDownloadPlatform,
    arch: Vec<SystemArch>,
) -> Result<Option<release_info::Release>, Box<dyn Error>> {
    let arch = arch.into_iter().map(Into::into).collect();
    let variables = release_info::Variables {
        version: Some(String::from(version)),
        limit: 1,
        platform: Some(vec![platform]),
        architecture: Some(arch),
    };
    let client = Client::new();
    let response_body = post_graphql::<ReleaseInfo, _>(&client, API_URL, variables)?;
    if let Some(response) = response_body.data {
        return match response.get_unity_releases.edges.first() {
            None => Ok(None),
            Some(edge) => Ok(Some(edge.node.release.clone())),
        };
    }
    Err("Couldn't retrieve version info".into())
}
