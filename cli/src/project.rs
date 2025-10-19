use crate::GlobalOpts;
use clap::Args;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::install::install_editor;

#[derive(Debug, Args)]
pub struct ProjectEditorVersionArgs {
    ///path to the project
    #[clap(long, short)]
    path: PathBuf,
}

#[derive(Debug, Args)]
pub struct ProjectOpenArgs {
    ///path to the project
    #[clap(long)]
    path: PathBuf,
    ///the module id. The followings are the available values depending on version. You can specify multiple values, separated by spaces.
    #[clap(long, num_args = 1..)]
    module: Option<Vec<String>>,
    ///active build target
    #[clap(long)]
    target: String,
    ///editor architecture (x86_64 or arm64)
    #[clap(long)]
    architecture: Option<String>,
}

#[derive(Debug, Args)]
pub struct ProjectExecuteArgs {
    ///path to the project
    #[clap(long)]
    path: PathBuf,
    ///method to execute
    #[clap(long)]
    method: String,
    ///the module id. The followings are the available values depending on version. You can specify multiple values, separated by spaces.
    #[clap(long, num_args = 1..)]
    module: Option<Vec<String>>,
    ///active build target
    #[clap(long)]
    target: String,
    ///license activation username
    #[clap(long)]
    username: String,
    ///license activation password
    #[clap(long)]
    password: String,
    ///license activation serial key
    #[clap(long)]
    serial: String,
    ///editor architecture (x86_64 or arm64)
    #[clap(long)]
    architecture: Option<String>,
    ///additional arguments for editor
    #[clap(allow_hyphen_values=true, last=true)]
    arguments: Vec<String>
}

#[derive(Debug, Args)]
pub struct ProjectCreateArgs {
    ///editor version to use for project creation
    #[clap(long, short)]
    version: String,
    ///path where the project should be created
    #[clap(long)]
    path: PathBuf,
    ///editor architecture (x86_64 or arm64)
    #[clap(long)]
    architecture: Option<String>,
}

pub fn editor_version(args: ProjectEditorVersionArgs, _global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    let editor_version = wrum_lib::projects::get_project_editor_version(args.path)?;
    println!("{}", editor_version);
    Ok(0)
}

pub fn open(args: ProjectOpenArgs, _global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    let editor_path = get_or_install_editor(&args.path, args.module, args.architecture)?;
    let executable_path = wrum_lib::system_info::get_editor_executable_path(editor_path);
    Command::new(executable_path.clone())
        .arg("-projectPath")
        .arg(&args.path)
        .arg("-buildTarget")
        .arg(&args.target)
        .status()?;
    Ok(0)
}

pub fn execute(args: ProjectExecuteArgs, _global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    let editor_path = get_or_install_editor(&args.path, args.module, args.architecture)?;
    let executable_path = wrum_lib::system_info::get_editor_executable_path(editor_path);
    let license_lock_file = wrum_lib::license::lock_license(&args.username)?;
    Command::new(executable_path.clone())
        .arg("-projectPath")
        .arg(&args.path)
        .arg("-buildTarget")
        .arg(&args.target)
        .arg("-logfile")
        .arg("-")
        .arg("-batchmode")
        .arg("-silent-crashes")
        .arg("-quit")
        .arg("-username")
        .arg(&args.username)
        .arg("-password")
        .arg(&args.password)
        .arg("-serial")
        .arg(&args.serial)
        .arg("-executeMethod")
        .arg(&args.method)
        .args(args.arguments)
        .status()?;
    let can_return_license = wrum_lib::license::release_license(&args.username, license_lock_file)?;
    if can_return_license {
        println!("Return license");
        Command::new(executable_path.clone())
            .arg("-projectPath")
            .arg(&args.path)
            .arg("-logfile")
            .arg("-")
            .arg("-batchmode")
            .arg("-nographics")
            .arg("-quit")
            .arg("-username")
            .arg(&args.username)
            .arg("-password")
            .arg(&args.password)
            .arg("-returnlicense")
            .status()?;
    } else {
        println!("License is still in use");
    }
    Ok(0)
}

pub fn create(args: ProjectCreateArgs, _global_opt: GlobalOpts) -> Result<i32, Box<dyn Error>> {
    install_editor(&args.version, None, args.architecture.clone(), false)?;
    let editor_path = wrum_lib::editors::get_installed_editor_path(&args.version, args.architecture.clone())?;
    let editor_path = match editor_path {
        Some(path) => path,
        None => return Err("Something went wrong. Failed to install and obtain an editor".into())
    };
    if let Some(parent) = args.path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    let executable_path = wrum_lib::system_info::get_editor_executable_path(editor_path);
    Command::new(executable_path)
        .arg("-createProject")
        .arg(&args.path)
        .status()?;
    Ok(0)
}

fn get_or_install_editor(project_path: impl AsRef<Path>, modules: Option<Vec<String>>, arch: Option<String>) -> Result<PathBuf, Box<dyn Error>> {
    let editor_version = wrum_lib::projects::get_project_editor_version(project_path)?;
    install_editor(&editor_version, modules, arch.clone(), true)?;
    let editor_path = wrum_lib::editors::get_installed_editor_path(&editor_version, arch)?;
    if editor_path.is_none() {
        return Err("Something went wrong. Failed to install and obtain an editor".into());
    }
    Ok(editor_path.unwrap())
}
