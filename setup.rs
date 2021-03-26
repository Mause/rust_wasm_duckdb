//! ```cargo
//! [package]
//! edition = "2018"
//! [dependencies]
//! zip = "*"
//! tokio = { version = "1", features = ["full"] }
//! reqwest = {version="*"}
//! octocrab = { version = "*" }
//! ```

use octocrab::models::repos::Release;
use std::io::Read;
use tokio::fs::File;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let release = octocrab::instance()
        .repos("cwida", "duckdb")
        .releases()
        .get_latest()
        .await
        .expect("latest");

    std::fs::create_dir_all("target")?;

    println!("Latest release: {}", &release.tag_name);
    tokio::try_join!(
        from_file(&release, "libduckdb-src.zip", "duckdb.hpp"),
        from_file(&release, "duckdb-wasm32-nothreads.zip", "duckdb.wasm")
    )?;

    Ok(())
}

async fn from_file(
    release: &Release,
    zip_filename: &str,
    inner_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = release
        .assets
        .iter()
        .find(|f| f.name == zip_filename)
        .expect("no sauce?")
        .browser_download_url
        .clone();

    println!("Downloading {}", zip_filename);
    let res = reqwest::get(url).await.expect("no zip?");

    let zip = res.bytes().await.expect("no bytes?").to_vec();
    println!("Downloaded {}", zip_filename);

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();

    let mut file = archive
        .by_name(inner_filename)
        .expect(format!("File {} not found", inner_filename).as_str());

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).expect("read_to_end");

    tokio::io::copy(
        &mut std::io::Cursor::new(contents),
        &mut File::create(format!("target/{}", inner_filename)).await?,
    )
    .await?;

    Ok(())
}
