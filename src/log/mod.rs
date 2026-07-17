use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    process::Command,
    sync::Mutex,
};

static LOGFILE: Mutex<Option<File>> = Mutex::new(None);

pub fn set_logfile(path: &str) -> io::Result<String> {
    let stamped = stamped_path(path);
    *LOGFILE.lock().unwrap() = Some(File::create(&stamped)?);
    Ok(stamped)
}

fn stamped_path(path: &str) -> String {
    insert_stamp(path, &timestamp())
}

fn insert_stamp(path: &str, stamp: &str) -> String {
    let path = Path::new(path);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("log");
    let stamped_name = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => format!("{stem}_{stamp}.{ext}"),
        None => format!("{stem}_{stamp}"),
    };
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            parent.join(stamped_name).to_string_lossy().into_owned()
        }
        _ => stamped_name,
    }
}

fn timestamp() -> String {
    Command::new("date")
        .arg("+%Y-%m-%dT%H-%M-%S%z")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown-time".to_string())
}

pub fn write_log(line: &str) {
    if let Some(file) = LOGFILE.lock().unwrap().as_mut() {
        let _ = writeln!(file, "{}", strip_ansi(line));
    }
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(character) = chars.next() {
        if character == '\x1b' {
            for next in chars.by_ref() {
                if next == 'm' {
                    break;
                }
            }
        } else {
            out.push(character);
        }
    }
    out
}

#[macro_export]
macro_rules! echo {
    ($quiet:expr, $($arg:tt)*) => {{
        let line = format!($($arg)*);
        if !$quiet {
            println!("{line}");
        }
        $crate::log::write_log(&line);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_stamp_appends_before_extension() {
        [
            ("run.log", "run_STAMP.log"),
            ("run", "run_STAMP"),
            ("archive.tar.gz", "archive.tar_STAMP.gz"),
            ("", "log_STAMP"),
        ]
        .iter()
        .for_each(|(path, expected)| assert_eq!(insert_stamp(path, "STAMP"), *expected));
    }

    #[test]
    fn insert_stamp_preserves_parent_directory() {
        [("logs", "run.log", "run_STAMP.log"), ("a/b", "run", "run_STAMP")]
            .iter()
            .for_each(|(dir, file, expected_name)| {
                let path = Path::new(dir).join(file);
                let expected = Path::new(dir).join(expected_name);
                assert_eq!(
                    insert_stamp(path.to_str().unwrap(), "STAMP"),
                    expected.to_str().unwrap(),
                );
            });
    }

    #[test]
    fn timestamp_is_filename_safe() {
        let stamp = timestamp();
        assert!(!stamp.is_empty());
        assert_eq!(stamp.matches(':').count(), 0);
        assert_eq!(stamp.matches('T').count(), 1);
    }
}
