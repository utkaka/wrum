use crate::system::call_hub_command;
use crate::GlobalOpts;
use clap::Args;
use std::error::Error;

#[derive(Debug, Args)]
pub struct InstallModulesArgs {
    ///version of the editor to add the module to - required
    #[clap(long, short)]
    version: String,
    ///the module id. The followings are the available values depending on version. You can specify multiple values, separated by spaces.
    #[clap(long, short, num_args = 1..)]
    module: Vec<String>,
    ///automatically installs all child modules of selected modules
    ///automatically installs all child modules of selected modules
    #[clap(long, group = "child", default_value_t = false)]
    cm: bool,
    ///automatically installs all child modules of selected modules
    #[clap(long, group = "child", default_value_t = false)]
    child_modules: bool,
    ///editor architecture to install (x86_64 or arm64)
    #[clap(long, short)]
    architecture: Option<String>
}

pub fn execute(args: InstallModulesArgs, global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    let include_children = args.cm || args.child_modules;
    if global_opt.hub {
        let mut hub_arguments = Vec::from_iter([String::from("install-modules"), String::from("--version"), args.version]);
        for module in args.module {
            hub_arguments.push(String::from("--module"));
            hub_arguments.push(module);
        }
        if include_children {
            hub_arguments.push(String::from("--childModules"));
        }
        match call_hub_command(hub_arguments) {
            Ok(status) => Ok(status.code().unwrap()),
            Err(error) => Err(error.into()),
        }
    } else {
        install_modules(&args.version, args.module, args.architecture, include_children)
    }
}

pub fn install_modules(version: &str, modules: Vec<String>, arch: Option<String>, include_children: bool) -> Result<i32, Box<dyn Error>> {
    let mut modules_in_args: Vec<String> = vec![];
    for module in modules {
        modules_in_args.append(&mut module.split(' ').map(str::to_string).collect());
    }
    wrum_lib::modules::install_modules(version, modules_in_args, arch.clone(), include_children)?;
    wrum_lib::workarounds::apply_bee_workaround(&version, arch)?;
    Ok(0)
}
