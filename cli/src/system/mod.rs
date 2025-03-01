use std::ffi::OsStr;
use std::io;
use std::process::ExitStatus;
use wrum_lib::editors;
use wrum_lib::live_api::latest_major_releases::UnityReleaseDownloadArchitecture;

#[cfg_attr(target_os = "macos", path = "os/macos.rs")]
#[cfg_attr(target_os = "windows", path = "os/windows.rs")]
#[cfg_attr(target_os = "linux", path = "os/linux.rs")]
mod os;

pub fn call_hub_command<I, S>(args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    os::call_hub_command(args)
}

pub fn get_installed_arch_string(arch: editors::info::SystemArch) -> Option<String> {
    os::get_installed_arch_string(arch)
}

pub fn get_release_arch_string(arch: UnityReleaseDownloadArchitecture) -> Option<String> {
    os::get_release_arch_string(arch)
}
