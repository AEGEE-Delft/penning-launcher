#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env::temp_dir,
    path::{Path, PathBuf},
};

use octocrab::models::repos::Release;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

mod run;

#[tokio::main]
async fn main() {
    let current_version = semver::Version::parse(CURRENT_VERSION).unwrap();

    let rate = octocrab::instance().ratelimit().get().await.unwrap().rate;
    
    if rate.remaining == 0 {
        println!("Not checking for updates, rate limit exceeded");
        run::launch(rate).await;
    } else {
        if let Ok(rel) = octocrab::instance()
            .repos("AEGEE-Delft", "penning-launcher")
            .releases()
            .get_latest()
            .await
        {
            let without_prefix = rel
                .tag_name
                .strip_prefix("v")
                .unwrap_or_else(|| rel.tag_name.as_str());
            let v = semver::Version::parse(without_prefix).unwrap();
            if v > current_version {
                println!("New version available: {}", rel.tag_name);
                println!("Downloading...");
                if let Some(new_exe) = update_self(&rel).await {
                    println!("Update successful, restarting...");
                    rerun_self(&new_exe);
                }
            } else {
                println!("You are using the latest version: {}", CURRENT_VERSION);
            }
        } else {
            println!("Failed to get latest release, no internet?");
        }
        run::launch(rate).await;
    }
}

async fn update_self(rel: &Release) -> Option<PathBuf> {
    rfd::AsyncMessageDialog::new()
        .set_level(rfd::MessageLevel::Info)
        .set_title("Downloading update")
        .set_description("Downloading the latest version of the launcher")
        .set_buttons(rfd::MessageButtons::Ok)
        .show()
        .await;
    // mac, windows, linux
    let file = match std::env::consts::OS {
        "linux" => "launcher-x86_64-unknown-linux-gnu.tar.gz",
        "macos" => "launcher-universal-apple-darwin.tar.gz",
        "windows" => "launcher-x86_64-pc-windows-msvc.zip",
        _ => {
            return None;
        }
    };
    let asset = rel.assets.iter().find(|a| a.name == file);
    if let Some(asset) = asset {
        let dir = temp_dir().join("penning-launcher-update");
        if dir.exists() {
            let _ = tokio::fs::remove_dir_all(&dir).await;
        }
        tokio::fs::create_dir(&dir).await.unwrap();
        println!("Dir created: {:?}", dir);

        let res = reqwest::get(asset.browser_download_url.as_str())
            .await
            .unwrap();
        let bytes = res.bytes().await.unwrap();

        extract_file(&bytes, &dir).unwrap();
        let file_name = match std::env::consts::OS {
            "linux" => "penning-launcher",
            "macos" => "penning-launcher",
            "windows" => "penning-launcher.exe",
            _ => {
                return None;
            }
        };
        let new_exe = dir.join(file_name);
        println!("New exe: {:?}", new_exe);
        return Some(new_exe);
    }

    None
}

fn rerun_self(path: &Path) {
    let exe = std::env::current_exe().unwrap();
    self_replace::self_replace(path).unwrap();
    std::fs::remove_file(path).unwrap();
    std::process::Command::new(exe)
        .spawn()
        .expect("Failed to restart");
    std::process::exit(0);
}

#[cfg(windows)]
fn extract_file(file: &[u8], target: &Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let _ = zip_extract::extract(Cursor::new(file), target, true);
    Ok(())
}

#[cfg(not(windows))]
fn extract_file(file: &[u8], target: &Path) -> std::io::Result<()> {
    use flate2::read::GzDecoder;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    archive.unpack(target)?;
    Ok(())
}
