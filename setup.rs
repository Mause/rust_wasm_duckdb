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
    println!("Starting");
    let release = octocrab::instance()
        .repos("cwida", "duckdb")
        .releases()
        .get_latest()
        .await
        .expect("latest");
    println!("Release: {:?}", &release);

    let url = release
        .assets
        .iter()
        .find(|f| f.name == "libduckdb-src.zip")
        .expect("no sauce?")
        .browser_download_url
        .clone();

    println!("url: {}", &url);

    let res = reqwest::get(url)
        .await
        .expect("no zip?");
    println!("res: {:?}", &res);

    let zip = res
        .bytes()
        .await
        .expect("no bytes?")
        .to_vec();

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();

    let mut file = archive
        .by_name("duckdb.wasm")
        .expect("File duckdb.wasm not found");

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).expect("read_to_end");

    std::fs::write("src/duckdb.wasm", contents).expect("Unable to write duckdb.wasm");

    Ok(())
}
