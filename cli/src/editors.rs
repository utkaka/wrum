use crate::system::call_hub_command;
use crate::{system, GlobalOpts};
use clap::Args;
use std::error::Error;
use std::path::PathBuf;
use wrum_lib::live_api::latest_major_releases::ItemLatestUnityReleaseDownloads;
use wrum_lib::{editors, live_api};

#[derive(Debug, Args)]
pub struct EditorsArgs {
    ///list of available releases and installed editors on your machine combined
    #[clap(long, short, default_value_t = false, group = "input")]
    all: bool,
    ///only list of available releases promoted by Unity
    #[clap(long, short, default_value_t = false, group = "input")]
    releases: bool,
    ///only list of installed editors on your machine
    #[clap(long, short, default_value_t = false, group = "input")]
    installed: bool,
    ///locating and associating an editor from a stipulated path
    #[clap(long)]
    add: Option<PathBuf>,
}

pub fn execute(args: EditorsArgs, global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    if let Some(path) = args.add {
        add_path(path, global_opt)
    } else {
        list_editors(args.releases || args.all, args.installed || args.all, global_opt)
    }
}

fn add_path(path: PathBuf, _global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    match call_hub_command(["editors", "--add", path.to_str().unwrap()]) {
        Ok(status) => Ok(status.code().unwrap()),
        Err(error) => Err(error.into()),
    }
}

fn list_editors(releases: bool, installed: bool, global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    if global_opt.hub {
        let command_option = match (releases, installed) {
            (false, true) => "--installed",
            (true, false) => "--releases",
            _ => "--all",
        };
        match call_hub_command(["editors", command_option]) {
            Ok(status) => Ok(status.code().unwrap()),
            Err(error) => Err(error.into()),
        }
    } else {
        match (releases, installed) {
            (false, true) => print_installed_editors(),
            (true, false) => print_major_releases(),
            _ => {
                print_major_releases()?;
                print_installed_editors()
            }
        }
    }
}

fn print_installed_editors() -> Result<i32, Box<dyn Error>> {
    let editors = editors::list_installed_editors()?;
    for editor in editors {
        match system::get_installed_arch_string(editor.arch) {
            None => {
                println!("{}, installed at {}", editor.version, editor.executable_path.to_str().unwrap());
            }
            Some(arch) => {
                println!("{} ({}), installed at {}", editor.version, arch, editor.executable_path.to_str().unwrap());
            }
        }
    }
    Ok(0)
}

fn print_major_releases() -> Result<i32, Box<dyn Error>> {
    let editors = live_api::get_major_release_list()?;
    for editor in editors {
        let latest_release = editor.item.latest_unity_release;
        let full_version = latest_release.version;
        for download in latest_release.downloads {
            match download {
                ItemLatestUnityReleaseDownloads::UnityReleaseHubDownload(download) => match system::get_release_arch_string(download.architecture) {
                    None => {
                        println!("{}", full_version);
                    }
                    Some(arch) => {
                        println!("{} ({})", full_version, arch);
                    }
                },
            }
        }
    }
    Ok(0)
}
