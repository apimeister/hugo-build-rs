use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{Read, Write},
};
use tar::Archive;

#[test]
fn sanitize_version_strips_hotfix_suffix() {
    assert_eq!(crate::sanitize_version("0.154.5-hf1"), "0.154.5");
}

#[test]
fn sanitize_version_keeps_release_version() {
    assert_eq!(crate::sanitize_version("0.154.5"), "0.154.5");
}

#[test]
fn fetch() {
    let url = "https://github.com/gohugoio/hugo/releases/download/v0.115.1/hugo_extended_0.115.1_darwin-universal.tar.gz";
    let result = reqwest::blocking::get(url).unwrap();
    let bytes = result.bytes().unwrap();
    let decompressor = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(decompressor);
    for entry in archive.entries().unwrap() {
        let mut file = entry.unwrap();
        println!("file: {:?}", file.path());
        println!("bytes: {:?}", file.size());
        let mut local_file = File::create("target/test-extract").unwrap();
        let mut bytes: Vec<u8> = vec![];
        let result = file.read_to_end(&mut bytes).unwrap();
        println!("read: {result} expected: {}", file.size());
        local_file.write_all(&bytes).unwrap();
    }
}

#[test]
fn lifecycle() {
    unsafe {
        std::env::set_var("OUT_DIR", "./target/hugo");
        std::fs::create_dir_all("./target/hugo").unwrap();
    }
    use std::path::Path;
    let source = Path::new("target/hugo-input");
    let output = Path::new("target/hugo-output");
    std::fs::create_dir_all(source).unwrap();
    std::fs::create_dir_all(output).unwrap();
    crate::init()
        .with_input(source.to_path_buf())
        .with_output(output.to_path_buf())
        .build()
        .unwrap();
}
