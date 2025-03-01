use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};
use wrum_lib::editors::info::SystemArch;
use wrum_lib::live_api::latest_major_releases::UnityReleaseDownloadArchitecture;

static HUB_PATH: &str = "/Applications/Unity Hub.app/Contents/MacOS/Unity Hub";
static LABEL_INTEL_ARCH: &str = "Intel";
static LABEL_SILICON_ARCH: &str = "Apple silicon";

pub fn call_hub_command<I, S>(args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(HUB_PATH).arg("--").arg("--headless").args(args).status()
}

pub fn get_installed_arch_string(arch: SystemArch) -> Option<String> {
    match arch {
        SystemArch::X86_64 => Some(String::from(LABEL_INTEL_ARCH)),
        SystemArch::ARM64 => Some(String::from(LABEL_SILICON_ARCH)),
    }
}

pub fn get_release_arch_string(arch: UnityReleaseDownloadArchitecture) -> Option<String> {
    match arch {
        UnityReleaseDownloadArchitecture::X86_64 => Some(String::from(LABEL_INTEL_ARCH)),
        UnityReleaseDownloadArchitecture::ARM64 => Some(String::from(LABEL_SILICON_ARCH)),
        UnityReleaseDownloadArchitecture::Other(_) => None,
    }
}
