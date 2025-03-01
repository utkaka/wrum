mod editors;
mod install;
mod install_modules;
mod install_path;
mod system;
mod project;

use clap::{Args, Parser, Subcommand};

/// Written in Rust Unity Manager
#[derive(Debug, Parser)]
#[clap(name = "wrum", version, trailing_var_arg=true)]
pub struct App {
    ///Global flags
    #[clap(flatten)]
    global_opts: GlobalOpts,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    ///list the releases and installed editors (alias: e)
    #[clap(alias("e"))]
    Editors(editors::EditorsArgs),
    ///set/get the path where the Unity editors will be installed (alias: ip)
    #[clap(alias("ip"))]
    InstallPath(install_path::InstallPathArgs),
    ///installs a new editor either from the releases list or archive (alias: i)
    #[clap(alias("i"))]
    Install(install::InstallArgs),
    ///download and install a module (e.g. build support) to an installed editor (alias: im)
    #[clap(alias("im"))]
    InstallModules(install_modules::InstallModulesArgs),
    ///print project's editor version
    #[clap(alias("pv"))]
    ProjectEditorVersion(project::ProjectEditorVersionArgs),
    ///print project's editor version
    #[clap(alias("po"))]
    OpenProject(project::ProjectOpenArgs),
    #[clap(alias("pe"))]
    ExecuteProject(project::ProjectExecuteArgs),
}

#[derive(Debug, Args)]
struct GlobalOpts {
    ///if possible, run the command via Unity Hub CLI
    #[clap(long, default_value_t = false)]
    hub: bool,
    ///does nothing, just for compatibility with Unity Hub CLI
    #[clap(long, default_value_t = true)]
    headless: bool,
    ///pass errors flag to Unity Hub CLI
    #[clap(long, default_value_t = false)]
    errors: bool,
}

fn main() {
    let args = App::parse();
    let global_opt = args.global_opts;
    let exit_code = match args.command {
        Command::InstallPath(args) => install_path::execute(args, global_opt),
        Command::Editors(args) => editors::execute(args, global_opt),
        Command::Install(args) => install::execute(args, global_opt),
        Command::InstallModules(args) => install_modules::execute(args, global_opt),
        Command::ProjectEditorVersion(args) => project::editor_version(args, global_opt),
        Command::OpenProject(args) => project::open(args, global_opt),
        Command::ExecuteProject(args) => project::execute(args, global_opt),
    };
    match exit_code {
        Ok(code) => {
            std::process::exit(code);
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
