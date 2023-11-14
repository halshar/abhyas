use rusqlite::ffi::{SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_UNIQUE};
use rusqlite::Connection;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// struct to carry the db connection
struct Db {
    conn: rusqlite::Connection,
}

impl Db {
    /// create a new connection
    fn new(conn: rusqlite::Connection) -> Self {
        Db { conn }
    }

    /// add new links
    fn add_link(&self, link: String) -> rusqlite::Result<()> {
        match self.conn.execute(
            "INSERT INTO links (link,solved_count,is_solved,is_skipped) VALUES (?1,?2,?3,?4)",
            (&link, 0, 0, 0),
        ) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                rusqlite::Error::SqliteFailure(err, _) => {
                    if err.code == rusqlite::ErrorCode::ConstraintViolation
                        && (err.extended_code == SQLITE_CONSTRAINT_PRIMARYKEY
                            || err.extended_code == SQLITE_CONSTRAINT_UNIQUE)
                    {
                        Err(rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error {
                                code: err.code,
                                extended_code: err.extended_code,
                            },
                            Some("the link should be unique".to_owned()),
                        ))
                    } else {
                        Err(rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error {
                                code: err.code,
                                extended_code: err.extended_code,
                            },
                            None,
                        ))
                    }
                }
                _ => Err(err),
            },
        }
    }
}

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

fn main() -> rusqlite::Result<()> {
    let _conn = Db::new(create_db_connection().unwrap());

    _conn.add_link("demo".to_owned())?;

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

    fn create_test_db(path: String) -> Db {
        let conn = Connection::open(&path).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS links (
                link            TEXT PRIMARY KEY,
                solved_count    INTEGER NOT NULL,
                is_solved       INTEGER NOT NULL,
                is_skipped      INTEGER NOT NULL
            )",
            (),
        )
        .unwrap();

        Db { conn }
    }

    fn delete_test_db(path: String) {
        let path = PathBuf::from(path);

        if path.exists() {
            fs::remove_file(&path).expect("Failed to remove the test db file");
        }
    }

    #[test]
    fn check_unique_insert() {
        let path = String::from("unique.db");
        let conn = create_test_db(path.clone());
        assert!(conn.add_link("https://test.com".to_owned()).is_ok());
        delete_test_db(path);
    }

    #[test]
    fn check_duplicate_insert() {
        let path = String::from("duplicate.db");
        let conn = create_test_db(path.clone());
        conn.add_link("https://test.com".to_owned()).unwrap();
        assert!(conn.add_link("https://test.com".to_owned()).is_err());
        delete_test_db(path);
    }
}
