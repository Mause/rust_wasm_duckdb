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
use octocrab::models::repos::Release;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let release = octocrab::instance()
        .repos("cwida", "duckdb")
        .releases()
        .get_latest()
        .await
        .expect("latest");

    std::fs::create_dir_all("target")?;
    from_file(&release, "libduckdb-src.zip", "duckdb.hpp").await?;
    from_file(&release, "duckdb-wasm32-nothreads.zip", "duckdb.wasm").await?;

    Ok(())
}

async fn from_file(release: &Release, zip_filename: &str, inner_filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = release
        .assets
        .iter()
        .find(|f| f.name == zip_filename)
        .expect("no sauce?")
        .browser_download_url
        .clone();

    let res = reqwest::get(url)
        .await
        .expect("no zip?");

    let zip = res
        .bytes()
        .await
        .expect("no bytes?")
        .to_vec();

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();

    let mut file = archive
        .by_name(inner_filename)
        .expect(format!("File {} not found", inner_filename).as_str());

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).expect("read_to_end");

    std::fs::write(format!("target/{}", inner_filename).as_str(), contents).expect(format!("Unable to write {}", inner_filename).as_str());
    
    Ok(())
}
