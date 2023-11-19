use crate::CustomErrors;
use rusqlite::Connection;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// create directory to store the db file
fn create_file() -> Result<PathBuf, CustomErrors> {
    let cache_dir = match dirs::cache_dir() {
        Some(value) => value,
        None => return Err(CustomErrors::CacheDirectoryNotFound),
    };

    let dir_name = &cache_dir.join("abhyas");
    match fs::create_dir_all(&dir_name) {
        Ok(_) => (),
        Err(_) => return Err(CustomErrors::CreateDirectoryFailed),
    };

    let file_name = &dir_name.join("abhyas.db");
    match OpenOptions::new().write(true).create(true).open(&file_name) {
        Ok(_) => (),
        Err(e) => return Err(CustomErrors::FileCreationFailed(e.to_string())),
    }

    Ok(file_name.to_path_buf())
}

/// create db connection
pub fn create_db_connection() -> Result<Connection, CustomErrors> {
    let file_name = create_file()?;

    let conn = match Connection::open(file_name) {
        Ok(value) => value,
        Err(_) => return Err(CustomErrors::DBConnectionFailed),
    };

    if conn
        .execute(
            "CREATE TABLE IF NOT EXISTS links (
            link            TEXT PRIMARY KEY,
            solved_count    INTEGER NOT NULL,
            is_solved       INTEGER NOT NULL,
            is_skipped      INTEGER NOT NULL
        )",
            (),
        )
        .is_err()
    {
        return Err(CustomErrors::DBQueryFailed);
    }

    Ok(conn)
}

pub fn show_green(msg: &str) -> () {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if let Err(_) = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))) {
        println!("{}", msg);
        return;
    }

    if let Err(_) = writeln!(&mut stdout, "{}", msg) {
        println!("{}", msg);
    }
}

pub fn show_red(msg: &str) -> () {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if let Err(_) = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))) {
        println!("{}", msg);
        return;
    }

    if let Err(_) = writeln!(&mut stdout, "{}", msg) {
        println!("{}", msg);
    }
}
