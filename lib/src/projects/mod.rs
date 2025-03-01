use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

pub fn get_project_editor_version(path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let mut version_file_path = path.as_ref().to_path_buf();
    version_file_path.push("ProjectSettings");
    version_file_path.push("ProjectVersion.txt");
    if !version_file_path.exists() {
        return Err("Path doesn't contain a valid project".into());
    }
    let file = File::open(version_file_path)?;
    let lines = io::BufReader::new(file).lines();
    for line in lines.map_while(Result::ok) {
        if line.starts_with("m_EditorVersion: ") {
            return Ok(String::from(&line[17..]));
        }
    }
    Err("Couldn't detect editor version".into())
}