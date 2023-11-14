use rusqlite::{Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// create db directory and file and return the path
fn create_file() -> std::io::Result<PathBuf> {
    // create cache directory if not present
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cache directory not found"))?;

    let dir_name = &cache_dir.join("abhyas");
    if !Path::new(&dir_name).exists() {
        fs::create_dir(&dir_name)?;
    }

    // create db file if not present
    let file_name = &dir_name.join("abhyas.db");
    if !Path::new(&file_name).exists() {
        fs::File::create(&file_name)?;
    }

    Ok(file_name.to_path_buf())
}

/// create the default table if it does not exist and return the connection
fn create_db_connection() -> io::Result<Connection> {
    let file_name = create_file()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error creating file: {}", e)))?;

    // create connection
    let conn = Connection::open(file_name).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Error while creating db connection: {}", e),
        )
    })?;

    // create default table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
            link            TEXT PRIMARY KEY,
            solved_count    INTEGER NOT NULL,
            is_solved       INTEGER NOT NULL,
            is_skipped      INTEGER NOT NULL
        )",
        (),
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Error while creating db table: {}", e),
        )
    })?;

    Ok(conn)
}

fn main() -> std::io::Result<()> {
    match create_db_connection() {
        Ok(_) => println!("finished iterating!"),
        Err(e) => println!("error: {:?}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn check_create_file() {
        let output = create_file();
        assert!(output.is_ok());
    }

    #[test]
    fn check_db_connection() {
        let output = create_db_connection();
        assert!(output.is_ok())
    }
}
