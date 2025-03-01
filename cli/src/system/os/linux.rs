use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};
use wrum_lib::editors::info::SystemArch;
use wrum_lib::live_api::latest_major_releases::UnityReleaseDownloadArchitecture;

static HUB_PATH: &str = "unityhub";

pub fn call_hub_command<I, S>(args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(HUB_PATH).arg("--headless").args(args).status()
}

pub fn get_installed_arch_string(_arch: SystemArch) -> Option<String> {
    None
}

pub fn get_release_arch_string(_arch: UnityReleaseDownloadArchitecture) -> Option<String> {
    None
}
