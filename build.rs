use std::process::Command;

fn main() {
    // Tell Cargo to re-run build.rs if any file in the src directory changes.
    // This helps to avoid "dirty" or "stale" builds.
    // Since the Rust compiler is very efficient, if one
    // does not update any code in main.rs, Cargo might
    // think that nothing has changed and build running
    // the build script, and leave an out-of-date timestamp.
    println!("cargo:rerun-if-changed=src");

    // Git commit hash and dirty status check
    if let Ok(commit_hash) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        && let Ok(hash) = String::from_utf8(commit_hash.stdout)
    {
        let mut commit_info = hash.trim().to_string();

        // Check if there are uncommitted changes
        if let Ok(status_output) = Command::new("git").args(["status", "--porcelain"]).output()
            && !status_output.stdout.is_empty()
        {
            commit_info.push('+');
        }
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_info);
    }

    // Build timestamp
    if let Ok(date_output) = Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%S%z") // // ISO 8601 format: YYYY-MM-DDTHH:MM:SS-offset
        .output()
        && let Ok(time) = String::from_utf8(date_output.stdout)
    {
        println!("cargo:rustc-env=BUILD_TIME={}", time.trim());
    }
}
