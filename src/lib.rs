use cpio_archive::CpioReader;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};
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

static VERSION: &str = std::env!("CARGO_PKG_VERSION");

fn binary_filename() -> &'static str {
    if cfg!(target_os = "windows") {
        "hugo.exe"
    } else {
        "hugo"
    }
}

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
    let url = match ARCH {
        "darwin-universal" => format!(
            "https://github.com/gohugoio/hugo/releases/download/v{VERSION}/hugo_extended_{VERSION}_darwin-universal.pkg"
        ),
        _ => format!(
            "https://github.com/gohugoio/hugo/releases/download/v{VERSION}/hugo_extended_{VERSION}_{ARCH}.tar.gz"
        ),
    };
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let mut binary_name = out_path.join(binary_filename());

    // check for already downloaded binary
    let mut binary_exists = false;
    let result = out_path.read_dir().expect("reading OUT_DIR");
    let expected_binary_name = binary_filename();
    for file in result {
        let entry = file.unwrap();
        if entry.file_name() == std::ffi::OsStr::new(expected_binary_name)
            && entry
                .file_type()
                .map(|file_type| file_type.is_file())
                .unwrap_or(false)
        {
            binary_exists = true;
            binary_name = entry.path();
        }
    }
    if !binary_exists {
        if ARCH == "darwin-universal" {
            binary_name = download_pkg(&url, out_path).unwrap();
        } else {
            binary_name = download_tar_gz(&url, out_path).unwrap();
        }
    }
    HugoBuilder {
        binary: binary_name,
        ..Default::default()
    }
}

fn download_tar_gz(url: &str, out_path: &Path) -> Result<PathBuf, std::io::Error> {
    // download fresh binary
    let result = reqwest::blocking::get(url).unwrap();
    let bytes = result.bytes().expect("downloading the hugo binary failed");
    let decompressor = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(decompressor);
    let mut binary_name = PathBuf::new();
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
            binary_name = target_file_name.clone();
            #[cfg(not(target_os = "windows"))]
            fix_permissions(&local_file);
        }
    }
    Ok(binary_name)
}

fn download_pkg(url: &str, out_path: &Path) -> Result<PathBuf, std::io::Error> {
    // download fresh binary
    let result = reqwest::blocking::get(url).unwrap();
    let bytes = result.bytes().expect("downloading the hugo binary failed");
    let mut cursor = std::io::Cursor::new(bytes);
    let mut archive = apple_xar::reader::XarReader::new(&mut cursor).unwrap();
    let archive_bytes = archive.get_file_data_from_path("Payload").unwrap().unwrap();

    let mut decompressor = GzDecoder::new(&archive_bytes[..]);
    let mut bytes2: Vec<u8> = vec![];
    decompressor.read_to_end(&mut bytes2).unwrap();
    let mut c = std::io::Cursor::new(bytes2);
    let mut reader = cpio_archive::odc::OdcReader::new(&mut c);
    let mut binary_name = PathBuf::new();
    loop {
        let entry = reader.read_next().unwrap();
        if let Some(x) = entry {
            let name = x.name();
            let file_path = Path::new(name);
            let is_binary = file_path.starts_with("hugo") || file_path.ends_with("hugo");
            let target_file_name = out_path.join(file_path);

            if x.file_size() > 0 {
                if let Some(parent) = target_file_name.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                let mut limit = std::io::Read::take(&mut reader, x.file_size());
                let mut out_file = File::create(&target_file_name).unwrap();
                std::io::copy(&mut limit, &mut out_file).unwrap();
                if is_binary {
                    binary_name = target_file_name.clone();
                    #[cfg(not(target_os = "windows"))]
                    fix_permissions(&out_file);
                }
            }
        } else {
            break;
        }
    }
    if binary_name.as_os_str().is_empty() {
        Err(std::io::Error::other("No binary found"))
    } else {
        Ok(binary_name)
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
