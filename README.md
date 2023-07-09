A wrapper around the hugo binary to proving building capabilities.

This crate downloads the hugo binaries on demand during build. So the first build needs connectivity to github.

The version number reflects the hugo version embedded.

# Usage

Add the following lines to you `build.rs` file.
This will build a hugo page from the `site` directory and put the output into the `target` directory.

```rust
use std::path::Path;

fn main() {
    let base = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let sources = Path::new(&base).join("site");
    let destination = Path::new(&base).join("target").join("site");
    println!("cargo:rerun-if-changed={}",sources.display());
    hugo_build::init()
        .with_input(sources)
        .with_output(destination)
        .build()
        .unwrap();
}
```