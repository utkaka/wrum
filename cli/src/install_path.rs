use crate::system::call_hub_command;
use crate::GlobalOpts;
use clap::Args;
use std::error::Error;
use std::path::PathBuf;
use wrum_lib::install::{get_install_path, set_secondary_install_path};

#[derive(Debug, Args)]
pub struct InstallPathArgs {
    ///returns the install path
    #[clap(long, short, default_value_t = true, group = "input")]
    get: bool,
    ///reset the install path to the default value
    #[clap(long, short, default_value_t = false, group = "input")]
    reset: bool,
    ///set the install path to the given path
    #[clap(long, short, group = "input")]
    set: Option<PathBuf>,
}

pub fn execute(args: InstallPathArgs, global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    if let Some(path) = args.set {
        set_path(path, global_opt)
    } else if args.reset {
        reset_path()
    } else {
        print_path(global_opt)
    }
}

fn set_path(path: PathBuf, global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    if global_opt.hub {
        match call_hub_command(["install-path", "--set", path.to_str().unwrap()]) {
            Ok(status) => Ok(status.code().unwrap()),
            Err(error) => Err(error.into()),
        }
    } else {
        match set_secondary_install_path(path, true) {
            Ok(_) => Ok(0),
            Err(error) => Err(error.into()),
        }
    }
}

fn reset_path() -> Result<i32, Box<dyn Error>> {
    match set_secondary_install_path(PathBuf::from(""), false) {
        Ok(_) => Ok(0),
        Err(error) => Err(error.into()),
    }
}

fn print_path(global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    if global_opt.hub {
        match call_hub_command(["install-path", "--get"]) {
            Ok(status) => Ok(status.code().unwrap()),
            Err(error) => Err(error.into()),
        }
    } else {
        match get_install_path() {
            Ok(path) => {
                println!("{}", path.to_str().unwrap());
                Ok(0)
            }
            Err(error) => Err(error.into()),
        }
    }
}
