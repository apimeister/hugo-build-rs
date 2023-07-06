A wrapper around the hugo binary to proving building capabilities.

This crate has the hugo binaries embedded, so no external dependencies are pulled during build.

The version number reflect the hugo version embedded.

# Usage

Add the following lines to you `build.rs` file.
This will build a hugo page from the `site` directory and put the output into the `target` directory.

```rust
fn main() {
    let base = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let sources = Path::new(&base).join("site");
    let destination = Path::new(&base).join("target").join("site");
    hugo_build::init()
        .with_input(sources)
        .with_output(destination)
        .build()
        .unwrap();
}
```