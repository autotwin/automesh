use std::process::Command;

fn main() {
    if let Ok(commit_hash) = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        && let Ok(hash) = String::from_utf8(commit_hash.stdout)
    {
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", hash);
    }
}
