use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src");
    if let Ok(commit_hash) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        && let Ok(hash) = String::from_utf8(commit_hash.stdout)
    {
        let mut commit_info = hash.trim().to_string();
        if let Ok(status_output) = Command::new("git").args(["status", "--porcelain"]).output()
            && !status_output.stdout.is_empty()
        {
            commit_info.push('+');
        }
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_info);
    }
    if let Ok(date_output) = Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%S%z")
        .output()
        && let Ok(time) = String::from_utf8(date_output.stdout)
    {
        println!("cargo:rustc-env=BUILD_TIME={}", time.trim());
    }
}
