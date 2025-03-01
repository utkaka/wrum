use crate::live_api::release_info::FileType;
use apple_xar::reader::XarReader;
use apple_xar::XarResult;
use cpio_reader::Mode;
use dmg::Attach;
use flate2::read::GzDecoder;
use regex::Regex;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::Permissions;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{fs, io};
use tempfile::TempDir;
use xz2::read::XzDecoder;

pub fn unpack(module_type: FileType, file_path: impl AsRef<Path>, target_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let file_path = file_path.as_ref();
    if !target_path.as_ref().exists() {
        fs::create_dir_all(&target_path)?;
    }

    let mut module_type = module_type;
    if let Some(extension) = file_path.extension() {
        module_type = match extension.to_ascii_lowercase().to_str() {
            Some("txt") => FileType::TEXT,
            Some("gz") => FileType::TAR_GZ,
            Some("xz") => FileType::TAR_XZ,
            Some("zip") => FileType::ZIP,
            Some("pkg") => FileType::PKG,
            Some("exe") => FileType::EXE,
            Some("po") => FileType::PO,
            Some("dmg") => FileType::DMG,
            Some("md") => FileType::MD,
            Some("pdf") => FileType::PDF,
            _ => module_type,
        }
    }

    match module_type {
        FileType::TEXT => unimplemented!(),
        FileType::TAR_GZ => unpack_tar_gz(file_path, target_path)?,
        FileType::TAR_XZ => unpack_tar_xz(file_path, target_path)?,
        FileType::ZIP => unpack_zip(file_path, target_path)?,
        FileType::PKG => unpack_pkg(file_path, target_path)?,
        FileType::EXE => unimplemented!(),
        FileType::PO => unpack_po(file_path, target_path)?,
        FileType::DMG => unpack_dmg(file_path, target_path)?,
        FileType::LZMA => unimplemented!(),
        FileType::LZ4 => unimplemented!(),
        FileType::MD => unimplemented!(),
        FileType::PDF => unimplemented!(),
        FileType::Other(_) => unimplemented!(),
    }

    Ok(())
}

pub fn move_files(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    copy_files(&from, to)?;
    fs::remove_dir_all(from)?;
    Ok(())
}
fn copy_files(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            copy_files(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else if entry_type.is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(entry.path())?, to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn unpack_po(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&destination_folder_path)?;

    let mut target_path = PathBuf::new();
    target_path.push(destination_folder_path);
    target_path.push(file_path.as_ref().file_name().unwrap());
    fs::copy(&file_path, target_path)?;
    fs::remove_file(file_path)?;
    Ok(())
}

fn unpack_zip(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let zip_file = fs::File::open(file_path)?;
    let mut zip = zip::ZipArchive::new(zip_file)?;
    zip.extract(destination_folder_path)?;
    Ok(())
}

fn unpack_dmg(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let dmg = Attach::new(file_path.as_ref()).with()?;
    let mount_path = &dmg.mount_point;
    for entry in fs::read_dir(mount_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("app")) {
            continue;
        }
        let re = Regex::new(r"/visual\s?studio.*\.app$/i")?;
        let mut target_path = PathBuf::new();
        target_path.push(destination_folder_path);
        if re.is_match(path.to_str().unwrap()) {
            target_path.push(path.file_name().unwrap());
        }
        copy_files(path, target_path)?;
        break;
    }

    Ok(())
}

fn unpack_tar_gz(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let bytes = decompress_gzip(file_path)?;
    unpack_tar(&bytes, destination_folder_path)?;
    Ok(())
}

fn unpack_tar_xz(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let bytes = decompress_xz(file_path)?;
    let mut destination_folder_path = PathBuf::from(destination_folder_path.as_ref());
    let playback_engines_path = ["Editor", "Data", "PlaybackEngines"].iter().collect::<PathBuf>();
    if destination_folder_path.to_str().unwrap_or("").contains(playback_engines_path.to_str().unwrap()) {
        while !destination_folder_path.ends_with(&playback_engines_path) {
            destination_folder_path.pop();
        }
        destination_folder_path.pop();
        destination_folder_path.pop();
        destination_folder_path.pop();
    }
    unpack_tar(&bytes, destination_folder_path)?;
    Ok(())
}

fn unpack_tar(tar_bytes: &[u8], destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let mut archive = tar::Archive::new(&*tar_bytes);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let mut entry_dest_path = PathBuf::new();
        entry_dest_path.push(&destination_folder_path);
        entry_dest_path.push(entry.path()?);
        entry.unpack(entry_dest_path)?;
    }
    Ok(())
}

fn unpack_pkg(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new().unwrap();
    unpack_xar(file_path, temp_dir.path())?;
    let payload_path = find_payload_file(temp_dir.path())?;
    let gzip_bytes = decompress_gzip(payload_path)?;
    unpack_cpio(&gzip_bytes, destination_folder_path)?;
    Ok(())
}

fn unpack_xar(file_path: impl AsRef<Path>, destination_folder_path: impl AsRef<Path>) -> XarResult<()> {
    let xar_file = fs::File::open(file_path)?;
    let mut xar = XarReader::new(xar_file)?;
    xar.unpack(destination_folder_path)
}

fn unpack_cpio(bytes: &[u8], destination_folder_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    for entry in cpio_reader::iter_files(bytes) {
        let mut entry_dest_path = PathBuf::new();
        entry_dest_path.push(&destination_folder_path);
        entry_dest_path.push(entry.name());
        if entry.mode().contains(Mode::DIRECTORY) {
            fs::create_dir_all(entry_dest_path)?;
        } else if entry.mode().contains(Mode::SYMBOLIK_LINK) {
            std::os::unix::fs::symlink(String::from_utf8(entry.file().into())?, &entry_dest_path)?;
        } else {
            fs::write(&entry_dest_path, entry.file())?;
            fs::set_permissions(&entry_dest_path, Permissions::from_mode(entry.mode().bits()))?;
        }
    }

    Ok(())
}

fn decompress_xz(file_path: impl AsRef<Path>) -> Result<Vec<u8>, Box<dyn Error>> {
    let xz_file = fs::File::open(file_path)?;
    let mut gzip_decoder = XzDecoder::new(xz_file);
    let mut decoded_bytes = Vec::new();
    gzip_decoder.read_to_end(&mut decoded_bytes)?;
    Ok(decoded_bytes)
}

fn decompress_gzip(file_path: impl AsRef<Path>) -> Result<Vec<u8>, Box<dyn Error>> {
    let gzip_file = fs::File::open(file_path)?;
    let mut gzip_decoder = GzDecoder::new(gzip_file);
    let mut decoded_bytes = Vec::new();
    gzip_decoder.read_to_end(&mut decoded_bytes)?;
    Ok(decoded_bytes)
}

fn find_payload_file(path: impl AsRef<Path>) -> Result<PathBuf, Box<dyn Error>> {
    let root = fs::read_dir(path);
    for child in root? {
        let mut entry = child?.path();
        if !entry.is_dir() || !entry.to_str().unwrap().ends_with("pkg.tmp") {
            continue;
        }
        entry.push("Payload");
        return Ok(entry);
    }
    Err("Couldn't find payload file".into())
}
