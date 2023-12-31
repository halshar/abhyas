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

    /// get total, completed, and skipped links count
    pub fn get_status(&self) -> Result<Option<(i32, i32, i32)>, CustomErrors> {
        let mut stmt = match self.conn.prepare(
            "SELECT
                COALESCE(COUNT(*), 0) AS total_links,
                COALESCE(SUM(CASE WHEN is_solved = 1 THEN 1 ELSE 0 END), 0) AS completed_links,
                COALESCE(SUM(CASE WHEN is_skipped = 1 THEN 1 ELSE 0 END), 0) AS skipped_links
            FROM links",
        ) {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        match stmt.query_row([], |row| {
            let total_links: i32 = row.get(0)?;
            let completed_links: i32 = row.get(1)?;
            let skipped_links: i32 = row.get(2)?;
            Ok((total_links, completed_links, skipped_links))
        }) {
            Ok((total_links, completed_links, skipped_links)) => {
                Ok(Some((total_links, completed_links, skipped_links)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(_) => Err(CustomErrors::Others(
                "Error: Something went wrong while checking the status".to_owned(),
            )),
        }
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
                        Err(CustomErrors::DuplicateLinkValue)
                    } else {
                        Err(CustomErrors::Others(
                            "Error: Something went wrong while inserting link".to_owned(),
                        ))
                    }
                }
                _ => Err(CustomErrors::Others(
                    "Error: Something went wrong while inserting link".to_owned(),
                )),
            },
        }
    }

    /// delete link from the db
    pub fn delete_link(&self, link: String) -> Result<(), CustomErrors> {
        match self
            .conn
            .execute("DELETE FROM links WHERE link = ?1", [&link])
        {
            Ok(_) => (),
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: Something went wrong while deleting the selected link".to_owned(),
                ))
            }
        }
        Ok(())
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

    pub fn get_searched_link_count(&self, link: &str) -> Result<Option<i32>, CustomErrors> {
        let mut stmt = match self.conn.prepare(
            "SELECT solved_count FROM links
            WHERE link = ?1;",
        ) {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        match stmt.query_row([&link], |row| {
            let solved_count: i32 = row.get(0)?;
            Ok(solved_count)
        }) {
            Ok(solved_count) => Ok(Some(solved_count)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(_) => Err(CustomErrors::Others(
                "Error: Something went wrong while fetching link count".to_owned(),
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

    pub fn get_all_links(&self) -> Result<Option<Vec<(String, i32)>>, CustomErrors> {
        let mut stmt = match self.conn.prepare("SELECT link, solved_count FROM links;") {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        let rows_iter = match stmt.query_map([], |row| {
            let link: String = row.get(0)?;
            let solved_count: i32 = row.get(1)?;
            Ok((link, solved_count))
        }) {
            Ok(val) => val,
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: While fetching all links".to_owned(),
                ))
            }
        };

        let mut links_solved_count_vec: Vec<(String, i32)> = vec![];
        for row in rows_iter {
            match row {
                Ok(val) => links_solved_count_vec.push(val),
                Err(_) => {
                    return Err(CustomErrors::Others(
                        "Error: While fetching unsolved link".to_owned(),
                    ))
                }
            };
        }

        if links_solved_count_vec.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links_solved_count_vec))
        }
    }

    /// returns only the completed links along with their solved_count
    pub fn get_completed_links(&self) -> Result<Option<Vec<(String, i32)>>, CustomErrors> {
        let mut stmt = match self
            .conn
            .prepare("SELECT link, solved_count FROM links where is_solved = 1;")
        {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        let rows_iter = match stmt.query_map([], |row| {
            let link: String = row.get(0)?;
            let solved_count: i32 = row.get(1)?;
            Ok((link, solved_count))
        }) {
            Ok(val) => val,
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: While fetching completed links".to_owned(),
                ))
            }
        };

        let mut links_completed_vec: Vec<(String, i32)> = vec![];
        for row in rows_iter {
            match row {
                Ok(val) => links_completed_vec.push(val),
                Err(_) => {
                    return Err(CustomErrors::Others(
                        "Error: While fetching completed link".to_owned(),
                    ))
                }
            };
        }

        if links_completed_vec.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links_completed_vec))
        }
    }

    /// returns only the skipped links along with their solved_count
    pub fn get_skipped_links(&self) -> Result<Option<Vec<(String, i32)>>, CustomErrors> {
        let mut stmt = match self
            .conn
            .prepare("SELECT link, solved_count from links WHERE is_skipped = 1;")
        {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        let rows_iter = match stmt.query_map([], |row| {
            let link: String = row.get(0)?;
            let solved_count: i32 = row.get(1)?;
            Ok((link, solved_count))
        }) {
            Ok(val) => val,
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: While fetching all skipped links".to_owned(),
                ))
            }
        };

        let mut skipped_links_vec: Vec<(String, i32)> = vec![];
        for row in rows_iter {
            match row {
                Ok(val) => skipped_links_vec.push(val),
                Err(_) => {
                    return Err(CustomErrors::Others(
                        "Error: While fetching all skipped links".to_owned(),
                    ))
                }
            };
        }

        if skipped_links_vec.is_empty() {
            Ok(None)
        } else {
            Ok(Some(skipped_links_vec))
        }
    }

    /// mark all skiped links as incomplete links
    pub fn skipped_to_incomplete(&self) -> Result<usize, CustomErrors> {
        self.conn
            .execute("UPDATE links SET is_skipped = 0 WHERE is_skipped = 1;", ())
            .map_err(|_| {
                CustomErrors::Others(
                    "Error: While trying to change all skipped links to incomplete".to_owned(),
                )
            })
    }

    /// mark all completed links as incomplete links
    pub fn completed_to_incomplete(&self) -> Result<usize, CustomErrors> {
        self.conn
            .execute("UPDATE links SET is_solved = 0 WHERE is_solved = 1;", ())
            .map_err(|_| {
                CustomErrors::Others(
                    "Error: While trying to change all completed links to incomplete".to_owned(),
                )
            })
    }

    /// add non-duplicate links from the file passed as argument
    pub fn insert_links_from_file(&self, links: &[String]) -> Result<usize, CustomErrors> {
        let values: String = links
            .iter()
            .map(|link| format!("('{}', 0, 0, 0)", link))
            .collect::<Vec<String>>()
            .join(",");

        let query = format!(
            "INSERT OR IGNORE INTO links (link, solved_count, is_solved, is_skipped) VALUES {}",
            values
        );

        let rows_updated_count = match self.conn.execute(&query, ()) {
            Ok(val) => val,
            Err(_) => {
                return Err(CustomErrors::Others(
                    "Error: Something went wrong while inserting links from file".to_owned(),
                ));
            }
        };

        Ok(rows_updated_count)
    }
}
