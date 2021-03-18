//! ```cargo
//! [package]
//! edition = "2018"
//! [dependencies]
//! zip = "*"
//! tokio = { version = "1", features = ["full"] }
//! reqwest = {version="*"}
//! octocrab = { version = "*" }
//! ```

use std::io::Read;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let release = octocrab::instance()
        .repos("cwida", "duckdb")
        .releases()
        .get_latest()
        .await
        .expect("latest");

    let url = release
        .assets
        .iter()
        .find(|f| f.name == "libduckdb-src.zip")
        .expect("no sauce?")
        .browser_download_url
        .clone();

    let zip = reqwest::get(url)
        .await
        .expect("no zip?")
        .bytes()
        .await
        .expect("no bytes?")
        .to_vec();

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();

    let mut file = archive
        .by_name("duckdb-wasm32-nothreads.zip")
        .expect("File duckdb-wasm32-nothreads.zip not found");

    let mut inner_file = zip::read::read_zipfile_from_stream(&mut file)
        .expect("Failed to open internal archive")
        .expect("Failed to read internal archive");

    let mut contents = Vec::new();
    inner_file.read_to_end(&mut contents).expect("read_to_end");

    std::fs::write("src/duckdb.wasm", contents).expect("Unable to write duckdb.wasm");

    Ok(())
}
