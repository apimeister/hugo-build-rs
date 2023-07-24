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
    /// path to the hugo binary
    binary: PathBuf,
    /// source directory
    input_path: Option<PathBuf>,
    /// target directory
    output_path: Option<PathBuf>,
}

#[cfg(target_os = "macos")]
static ARCH: &str = "darwin-universal";
#[cfg(target_os = "linux")]
static ARCH: &str = "Linux-64bit";
#[cfg(target_os = "windows")]
static ARCH: &str = "windows-amd64";

static VERSION: &str = "0.115.4";

#[cfg(not(target_os = "windows"))]
fn fix_permissions(local_file: &File) {
    //set permissions
    use std::os::unix::prelude::PermissionsExt;
    let permissions = std::fs::Permissions::from_mode(0o755);
    local_file.set_permissions(permissions).unwrap();
}

/// initialises a hugo build
/// 
/// fetches the binary from github if required
pub fn init() -> HugoBuilder {
    // fetch binary from github
    let url = format!("https://github.com/gohugoio/hugo/releases/download/v{VERSION}/hugo_extended_{VERSION}_{ARCH}.tar.gz");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let release_profile = std::env::var("PROFILE").unwrap();
    let path = format!("{manifest_dir}/target/{release_profile}");
    let out_path = Path::new(&path);
    let mut binary_name = out_path.join("hugo");

    // check for already downloaded binary
    let hugo_bins = out_path.read_dir().expect("reading OUT_DIR")
    .filter(|a | if let Ok(f) = a { println!("{:?}",f); f.file_name().eq("hugo") } else {false} )
    .count();
    if hugo_bins == 0 {
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
    } else {
        let version = Command::new(&binary_name)
            .arg("version")
            .output().unwrap();
        let version = std::str::from_utf8(&version.stdout).unwrap();
        let version = version.strip_prefix("hugo v").unwrap().split_once('-').unwrap().0;
        println!("{version}");
        println!("{VERSION}");
        if version != VERSION { 
            println!("cargo:warning=Hugo Build Script - Used Version: {version} - hugo-build-rs recommended Version: {VERSION}");
            println!("cargo:warning=run \"cargo clean\" to remove cached version and download recommended one");
        }
    }



    HugoBuilder {
        binary: binary_name,
        ..Default::default()
    }
}

impl HugoBuilder {
    /// defines source directory for the hugo build
    pub fn with_input(self, path: PathBuf) -> HugoBuilder {
        let mut cpy = self;
        cpy.input_path = Some(path);
        cpy
    }
    /// defines target directory for the hugo build
    pub fn with_output(self, path: PathBuf) -> HugoBuilder {
        let mut cpy = self;
        cpy.output_path = Some(path);
        cpy
    }
    pub fn build(self) -> Result<Output, std::io::Error> {
        let base = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let input = match self.input_path {
            None => {
                println!("cargo:warning=no input path set, using ./site");
                Path::new(&base).join("site")
            }
            Some(val) => val,
        };
        let output = match self.output_path {
            None => {
                println!("cargo:warning=no output path set, using ./target/site");
                Path::new(&base).join("target").join("site")
            }
            Some(val) => val,
        };
        Command::new(self.binary)
            .arg("-s")
            .arg(input)
            .arg("-d")
            .arg(output)
            .output()
    }
}
