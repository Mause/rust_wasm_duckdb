#![feature(try_trait)]

use shellexpand::full;

use std::ops::Deref;
use which::which;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let OUT = "pkg/libduckdb.js";
    if std::path::Path::new(OUT).exists() {
        return Ok(());
    }

    let emcc_path =
        which("em++").expect("Couldn't find em++, is the emscripten environment activated?");
    let emar_path =
        which("emar").expect("Couldn't find emar, is the emscripten environment activated?");

    cc::Build::new()
        .cpp(true)
        .flag("-Wno-everything")
        .define("DUCKDB_BUILD_LIBRARY", "1")
        .file(
            full("~/duckdb/src/amalgamation/duckdb.cpp")?
                .deref()
                .to_owned(),
        )
        .cpp_link_stdlib(None)
        .compiler(emcc_path)
        .archiver(emar_path)
        .compile(OUT);

    Ok(())
}
