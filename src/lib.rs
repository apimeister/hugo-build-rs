use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use flate2::read::GzDecoder;
use tar::Archive;

#[cfg(test)]
mod test;

#[derive(Debug, Default, Clone)]
pub struct HugoBuilder {
    binary: PathBuf,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
}

#[cfg(target_os = "macos")]
static ARCH: &str = "darwin-universal";
#[cfg(target_os = "linux")]
static ARCH: &str = "Linux-64bit";
#[cfg(target_os = "windows")]
static ARCH: &str = "windows-amd64";

static VERSION: &str = "0.115.1";

#[cfg(not(target_os = "windows"))]
fn fix_permissions(local_file: &File) {
    //set permissions
    use std::os::unix::prelude::PermissionsExt;
    let permissions = std::fs::Permissions::from_mode(0o755);
    local_file.set_permissions(permissions).unwrap();
}

pub fn init() -> HugoBuilder {
    // fetch binary from github
    let url = format!("https://github.com/gohugoio/hugo/releases/download/v{VERSION}/hugo_extended_{VERSION}_{ARCH}.tar.gz");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let mut binary_name = out_path.join("hugo");

    // check for already downloaded binary
    let mut binary_exists = false;
    let result = out_path.read_dir().expect("reading OUT_DIR");
    for file in result {
        let entry = file.unwrap();
        if entry.file_name().to_string_lossy().contains("hugo") {
            binary_exists = true;
            binary_name = entry.path();
        }
    }
    if !binary_exists {
        // download fresh binary
        let result = reqwest::blocking::get(url).unwrap();
        let bytes = result.bytes().unwrap();
        let decompressor = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(decompressor);
        for entry in archive.entries().unwrap() {
            let mut file = entry.unwrap();
            let file_path = file.path().unwrap();
            let is_binary = file_path.starts_with("hugo");
            let target_file_name = out_path.join(&file_path);
            let mut bytes: Vec<u8> = vec![];
            _ = file.read_to_end(&mut bytes).unwrap();
            let mut local_file = File::create(target_file_name.clone()).unwrap();
            local_file.write_all(&bytes).unwrap();
            if is_binary {
                binary_name = out_path.join(target_file_name.clone());
                #[cfg(not(target_os = "windows"))]
                fix_permissions(&local_file);
            }
        }
    }
    HugoBuilder {
        binary: binary_name,
        ..Default::default()
    }
}

impl HugoBuilder {
    pub fn with_input(self, path: PathBuf) -> HugoBuilder {
        let mut cpy = self;
        cpy.input_path = Some(path);
        cpy
    }
    pub fn with_output(self, path: PathBuf) -> HugoBuilder {
        let mut cpy = self;
        cpy.output_path = Some(path);
        cpy
    }
    pub fn build(self) -> Result<Output, std::io::Error> {
        Command::new(self.binary)
            .arg("-s")
            .arg(self.input_path.unwrap())
            .arg("-d")
            .arg(self.output_path.unwrap())
            .output()
    }
}
