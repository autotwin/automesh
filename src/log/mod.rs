use std::{
    fs::File,
    io::{self, Write},
    sync::Mutex,
};

static LOGFILE: Mutex<Option<File>> = Mutex::new(None);

pub fn set_logfile(path: &str) -> io::Result<()> {
    *LOGFILE.lock().unwrap() = Some(File::create(path)?);
    Ok(())
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
