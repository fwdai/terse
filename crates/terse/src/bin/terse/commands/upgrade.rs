use crate::utils::exit_err;

pub fn run_upgrade() {
    let current = env!("CARGO_PKG_VERSION");
    eprintln!("Current version: {}", current);
    eprintln!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("fwdai")
        .repo_name("terse")
        .bin_name("terse")
        .show_download_progress(true)
        .current_version(current)
        .build()
        .unwrap_or_else(|e| exit_err(&format!("failed to check for updates: {}", e), 1))
        .update()
        .unwrap_or_else(|e| exit_err(&format!("upgrade failed: {}", e), 1));

    match status {
        self_update::Status::UpToDate(v) => eprintln!("Already up to date (v{})", v),
        self_update::Status::Updated(v)  => eprintln!("Updated to v{}", v),
    }
}
