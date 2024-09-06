#![windows_subsystem = "windows"]

use octocrab::models::repos::Release;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let current_version = semver::Version::parse(CURRENT_VERSION).unwrap();

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
            update_self(&rel).await;
            rerun_self();
        } else {
            println!("You are using the latest version: {}", CURRENT_VERSION);
        }
        update_self(&rel).await;
    } else {
        println!("Failed to get latest release, no internet?");
    }

}

async fn update_self(rel: &Release) {
    // rfd::AsyncMessageDialog::new()
    //     .set_level(rfd::MessageLevel::Info)
    //     .set_title("Downloading update")
    //     .set_description("Downloading the latest version of the launcher")
    //     .set_buttons(rfd::MessageButtons::Ok)
    //     .show()
    //     .await;

    for asset in &rel.assets {
        println!("Downloading asset: {}", asset.name);
    }
}

fn rerun_self() {
    std::process::Command::new(std::env::current_exe().unwrap())
        .spawn()
        .expect("Failed to restart");
    std::process::exit(0);
}
