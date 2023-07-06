use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{Read, Write},
};
use tar::Archive;

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
        let mut local_file = File::create("1").unwrap();
        let mut bytes: Vec<u8> = vec![];
        let result = file.read_to_end(&mut bytes).unwrap();
        println!("read: {result} expected: {}", file.size());
        local_file.write_all(&bytes).unwrap();
    }
}
