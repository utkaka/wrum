use crate::system::call_hub_command;
use crate::GlobalOpts;
use clap::Args;
use std::error::Error;
use crate::install_modules::install_modules;

#[derive(Debug, Args)]
pub struct InstallArgs {
    ///editor version to be installed (e.g. 2019.1.11f1) - required
    #[clap(long, short)]
    version: String,
    ///changeset of the editor if it is not in the release list (e.g. 9b001d489a54) - required if the version is not in the releases
    #[clap(long, short)]
    changeset: Option<String>,
    ///the module id. The followings are the available values depending on version. You can specify multiple values, separated by spaces.
    #[clap(long, short, num_args = 1..)]
    module: Option<Vec<String>>,
    ///automatically installs all child modules of selected modules
    #[clap(long, group = "child", default_value_t = false)]
    cm: bool,
    ///automatically installs all child modules of selected modules
    #[clap(long, group = "child", default_value_t = false)]
    child_modules: bool,
    ///editor architecture to install (x86_64 or arm64)
    #[clap(long, short)]
    architecture: Option<String>,
}

pub fn execute(args: InstallArgs, global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    let include_child_modules = args.cm || args.child_modules;
    if global_opt.hub {
        let mut hub_arguments = Vec::from_iter([String::from("install"), String::from("--version"), args.version]);
        if let Some(changeset) = args.changeset {
            hub_arguments.push(String::from("--changeset"));
            hub_arguments.push(changeset);
        }
        if let Some(modules) = args.module {
            for module in modules {
                hub_arguments.push(String::from("--module"));
                hub_arguments.push(module);
            }
        }

        if include_child_modules {
            hub_arguments.push(String::from("--childModules"));
        }
        if let Some(architecture) = args.architecture {
            hub_arguments.push(String::from("--architecture"));
            hub_arguments.push(architecture);
        }
        match call_hub_command(hub_arguments) {
            Ok(status) => Ok(status.code().unwrap()),
            Err(error) => Err(error.into()),
        }
    } else {
        install_editor(&args.version, args.module, args.architecture, include_child_modules)
    }
}

pub fn install_editor(version: &str, modules: Option<Vec<String>>, arch: Option<String>, include_children: bool) -> Result<i32, Box<dyn Error>> {
    wrum_lib::editors::install_editor(version, arch.clone())?;
    if let Some(modules) = modules {
        install_modules(version, modules, arch, include_children)?;
    } else {
        wrum_lib::workarounds::apply_bee_workaround(&version, arch)?;
    }
    Ok(0)
}
