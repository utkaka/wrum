use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const CHUNK_SIZE: u32 = 10485760;

struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

struct PartialRangeIterHeader {
    header: HeaderValue,
    bytes: u64,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self, Box<dyn Error>> {
        if buffer_size == 0 {
            Err("invalid buffer_size, give a value greater than zero.")?;
        }
        Ok(PartialRangeIter { start, end, buffer_size })
    }
}

impl Iterator for PartialRangeIter {
    type Item = PartialRangeIterHeader;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some(PartialRangeIterHeader {
                header: HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1)).expect("string provided by format!"),
                bytes: self.start,
            })
        }
    }
}

pub fn download(url: &str, module_id: &str, editor_path: impl AsRef<Path>) -> Result<PathBuf, Box<dyn Error>> {
    let mut download_path = PathBuf::new();
    download_path.push(editor_path);
    download_path.push("downloads");
    fs::create_dir_all(&download_path)?;

    let client = reqwest::blocking::Client::new();
    let response = client.head(url).send()?;
    let length = response.headers().get(CONTENT_LENGTH).ok_or("response doesn't include the content length")?;
    let length = u64::from_str(length.to_str()?).map_err(|_| "invalid Content-Length header")?;

    let download_url = response.url();
    let mut filename = download_url.path().split('/').last().unwrap();
    if filename.is_empty() {
        filename = module_id;
    }
    let mut output_file_path = PathBuf::from(download_path);
    output_file_path.push(filename);

    let mut output_file: File;
    let mut start_byte = 0;
    match output_file_path.exists() {
        true => {
            output_file = File::options().read(true).write(true).append(true).open(&output_file_path)?;
            start_byte = output_file_path.metadata()?.len();
        }
        false => {
            output_file = File::create(&output_file_path)?;
        }
    }

    for range in PartialRangeIter::new(start_byte, length - 1, CHUNK_SIZE)? {
        println!("Downloading {}: {:.2}%.", module_id, (range.bytes as f32) / (length as f32) * 100.0);
        let mut response = client.get(url).header(RANGE, range.header).send()?;

        let status = response.status();
        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
            return Err("Unexpected server response".into());
        }
        std::io::copy(&mut response, &mut output_file)?;
    }
    Ok(output_file_path)
}
