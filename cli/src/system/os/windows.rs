use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};

static HUB_PATH: &str = "C:\\Program Files\\Unity Hub>Unity Hub.exe";

pub fn call_hub_command<I, S>(args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(HUB_PATH).arg("--").arg("--headless").args(args).status()
}

pub fn get_arch_string(arch: SystemArch) -> Option<String> {
    None
}

pub fn get_release_arch_string(arch: UnityReleaseDownloadArchitecture) -> Option<String> {
    None
}
