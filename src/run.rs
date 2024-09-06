use std::path::Path;

use chrono::Local;
use octocrab::models::Rate;

use crate::extract_file;

pub async fn launch(rate: Rate) {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("penning-launcher");
    println!("launching penning-helper");
    if rate.remaining == 0 {
        if data_dir.join(helper_exe_name()).exists() {
            println!("launching helper");
            let _ = std::process::Command::new(data_dir.join(helper_exe_name()))
                .spawn()
                .unwrap()
                .wait();
            return;
        } else {
            let date = chrono::DateTime::from_timestamp(rate.reset as i64, 0)
                .unwrap()
                .with_timezone(&Local)
                .naive_local();

            rfd::AsyncMessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Rate limit exceeded")
                .set_description(format!(
                    "You have exceeded the rate limit for GitHub API, please try again at {}",
                    date
                ))
                .set_buttons(rfd::MessageButtons::Ok)
                .show()
                .await;
        }
    } else {
        let helper_exe = data_dir.join(helper_exe_name());
        let _ = update_helper(&data_dir).await;
        if helper_exe.exists() {
            println!("launching helper");
            std::process::Command::new(helper_exe).spawn().unwrap();
        } else {
            rfd::AsyncMessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Failed to launch helper")
                .set_description("Failed to launch helper, please try again later")
                .set_buttons(rfd::MessageButtons::Ok)
                .show()
                .await;
        }
    }
}

fn helper_exe_name() -> &'static str {
    match std::env::consts::OS {
        "linux" => "penning-helper-interface",
        "macos" => "penning-helper-interface",
        "windows" => "penning-helper-interface.exe",
        _ => {
            return "penning-helper-interface.exe";
        }
    }
}

async fn update_helper(base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(rel) = octocrab::instance()
        .repos("AEGEE-Delft", "penning-helper")
        .releases()
        .get_latest()
        .await
    {
        println!("latest helper version: {}", rel.tag_name);
        let remote_version = semver::Version::parse(
            rel.tag_name
                .strip_prefix("v")
                .unwrap_or_else(|| rel.tag_name.as_str()),
        )?;
        let local_version = get_local_version(&base_path.join("version.json"));
        if local_version < remote_version {
            rfd::AsyncMessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_title("Downloading update")
                .set_description("Downloading the latest version of penning-helper")
                .set_buttons(rfd::MessageButtons::Ok)
                .show()
                .await;
            let asset = rel
                .assets
                .iter()
                .find(|a| a.name == asset_name())
                .ok_or("No asset found")?;
            let response = reqwest::get(asset.browser_download_url.as_str()).await?;
            extract_file(&response.bytes().await.unwrap(), base_path)?;
            let file = std::fs::File::create(base_path.join("version.json"))?;
            serde_json::to_writer(&file, &remote_version)?;

            rfd::AsyncMessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_title("Update successful")
                .set_description("The latest version of penning-helper has been downloaded")
                .set_buttons(rfd::MessageButtons::Ok)
                .show()
                .await;
        } else {
            println!("You are using the latest version: {}", local_version);
        }
    }

    Ok(())
}

fn get_local_version(file: &Path) -> semver::Version {
    if let Ok(f) = std::fs::File::open(file) {
        serde_json::from_reader(f).unwrap_or_else(|_| semver::Version::new(0, 0, 0))
    } else {
        semver::Version::new(0, 0, 0)
    }
}

fn asset_name() -> &'static str {
    match std::env::consts::OS {
        "linux" => "penning-helper-x86_64-unknown-linux-gnu.tar.gz",
        "macos" => "penning-helper-universal-apple-darwin.tar.gz",
        "windows" => "penning-helper-x86_64-pc-windows-msvc.zip",
        _ => "penning-helper-x86_64-pc-windows-msvc.zip",
    }
}
