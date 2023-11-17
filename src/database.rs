use crate::CustomErrors;
use rusqlite::ffi::{SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_UNIQUE};

/// struct to carry the db connection
pub struct Db {
    conn: rusqlite::Connection,
}

impl Db {
    /// create a new connection
    pub fn new(conn: rusqlite::Connection) -> Self {
        Db { conn }
    }

    /// add new links into the db
    pub fn add_link(&self, link: String) -> Result<(), CustomErrors> {
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
                        return Err(CustomErrors::DuplicateLinkValue);
                    } else {
                        return Err(CustomErrors::Others(
                            "Error: Something went wrong while inserting link".to_owned(),
                        ));
                    }
                }
                _ => {
                    return Err(CustomErrors::Others(
                        "Error: Something went wrong while inserting link".to_owned(),
                    ))
                }
            },
        }
    }

    pub fn get_links(&self) -> Result<Vec<String>, CustomErrors> {
        let mut stmt = match self.conn.prepare("SELECT link FROM links") {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        let rows = match stmt.query_map([], |row| row.get(0)) {
            Ok(val) => val,
            Err(e) => match e {
                rusqlite::Error::InvalidColumnName(e) => {
                    return Err(CustomErrors::InvalidColumnName(e))
                }
                _ => {
                    return Err(CustomErrors::Others(
                        "Error: Something went wrong while reading all links".to_owned(),
                    ))
                }
            },
        };

        let mut links: Vec<String> = Vec::new();
        for link_result in rows {
            links.push(link_result.unwrap())
        }
        Ok(links)
    }

    pub fn get_single_link(&self) -> Result<Option<(String, i32)>, CustomErrors> {
        let mut stmt = match self.conn.prepare(
            "SELECT link, solved_count FROM links
            WHERE is_solved = 0 AND is_skipped = 0
            LIMIT 1;",
        ) {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        match stmt.query_row([], |row| {
            let link: String = row.get(0)?;
            let solved_count: i32 = row.get(1)?;
            Ok((link, solved_count))
        }) {
            Ok((link, solved_count)) => Ok(Some((link, solved_count))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(_) => Err(CustomErrors::Others(
                "Error: While fetching unsolved link".to_owned(),
            )),
        }
    }

    pub fn mark_as_complete(&self, link: &str) -> Result<(), CustomErrors> {
        match self.conn.execute(
            "UPDATE links
            SET solved_count = solved_count + 1, is_solved = 1
            WHERE link = ?1;",
            [&link],
        ) {
            Ok(_) => (),
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: Something went wrong while marking the given link as complete"
                        .to_owned(),
                ))
            }
        };

        Ok(())
    }

    pub fn skip_link(&self, link: &str) -> Result<(), CustomErrors> {
        match self.conn.execute(
            "
            UPDATE links
            SET is_skipped = 1
            WHERE link = ?1;",
            [&link],
        ) {
            Ok(_) => (),
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: Something went wrong while skipping the link".to_string(),
                ))
            }
        };

        Ok(())
    }
}