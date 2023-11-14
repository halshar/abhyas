use rusqlite::ffi::{SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_UNIQUE};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

enum CustomErros {
    CacheDirectoryNotFound,
    CreateDirectoryFailed,
    CreateDBFileFailed,
    DBConnectionFailed,
    DBQueryFailed,
    DuplicateLinkValue,
    Others(String),
}

/// struct to carry the db connection
struct Db {
    conn: rusqlite::Connection,
}

impl Db {
    /// create a new connection
    fn new(conn: rusqlite::Connection) -> Self {
        Db { conn }
    }

    /// add new links into the db
    fn add_link(&self, link: String) -> Result<(), CustomErros> {
        match self.conn.execute(
            "INSERT INTO links (link,solved_count,is_solved,is_skipped) VALUES (?1,?2,?3,?4)",
            (&link, 0, 0, 0),
        ) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                rusqlite::Error::SqliteFailure(err, _) => {
                    if err.code == rusqlite::ErrorCode::ConstraintViolation
                        && (err.extended_code == SQLITE_CONSTRAINT_PRIMARYKEY
                            || err.extended_code == SQLITE_CONSTRAINT_UNIQUE)
                    {
                        return Err(CustomErros::DuplicateLinkValue);
                    } else {
                        return Err(CustomErros::Others(
                            "Error: Something went wrong while inserting link".to_owned(),
                        ));
                    }
                }
                _ => {
                    return Err(CustomErros::Others(
                        "Error: Something went wrong while inserting link".to_owned(),
                    ))
                }
            },
        }
    }
}

/// create directory to store the db file
fn create_file() -> Result<(), CustomErros> {
    let cache_dir = match dirs::cache_dir() {
        Some(value) => value,
        None => return Err(CustomErros::CacheDirectoryNotFound),
    };

    let dir_name = &cache_dir.join("abhyas");
    if Path::new(&dir_name).try_exists().is_err() {
        if fs::create_dir(&dir_name).is_err() {
            return Err(CustomErros::CreateDirectoryFailed);
        }
    }

    let file_name = &dir_name.join("abhyas.db");
    if Path::new(&file_name).try_exists().is_err() {
        if fs::File::create(&file_name).is_err() {
            return Err(CustomErros::CreateDBFileFailed);
        }
    }

    Ok(())
}

/// create db connection
fn create_db_connection() -> Result<Connection, CustomErros> {
    let file_name = create_file()?;

    let conn = match Connection::open(file_name) {
        Ok(value) => value,
        Err(e) => return Err(CustomErros::DBConnectionFailed),
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
        return Err(CustomErros::DBQueryFailed);
    }

    Ok(conn)
}

fn main() -> rusqlite::Result<()> {
    let _conn = Db::new(create_db_connection().unwrap());

    _conn.add_link("demo".to_owned())?;

    Ok(())
}

fn main() {
    let _conn = match create_db_connection() {
        Ok(value) => value,
        Err(e) => match e {
            CustomErros::CacheDirectoryNotFound => {
                println!("Error: The cache directory was not found")
            }
            CustomErros::CreateDirectoryFailed => {
                println!("Error: Couldn't create the db directory")
            }
            CustomErros::CreateDBFileFailed => println!("Error: Couldn't create the db file"),
            CustomErros::DBConnectionFailed => println!("Error: DB connection failed"),
            CustomErros::DBQueryFailed => println!("Error: DB query failed"),
            _ => unreachable!(),
        },
    };

    let db = Db::new(_conn);

    show_options(db);
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
