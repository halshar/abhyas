use inquire::{required, validator::Validation, Select, Text};
use rusqlite::ffi::{SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_UNIQUE};
use rusqlite::Connection;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;

#[derive(Debug)]
enum CustomErrors {
    CacheDirectoryNotFound,
    CreateDirectoryFailed,
    FileCreationFailed(String),
    DBConnectionFailed,
    DBQueryFailed,
    DuplicateLinkValue,
    StatementFailed,
    InvalidColumnName(String),
    OperationCanceled,
    OperationInterrupted,
    Others(String),
    Exit,
}

enum UserOptions {
    GetLink,
    AddLink,
    Exit,
}

enum InnerOptions {
    MarkAsComplete,
    SkipAndNext,
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
    fn add_link(&self, link: String) -> Result<(), CustomErrors> {
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

    fn get_links(&self) -> Result<Vec<String>, CustomErrors> {
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

    fn get_single_link(&self) -> Result<Option<String>, CustomErrors> {
        let mut stmt = match self.conn.prepare(
            "SELECT link FROM links 
        WHERE is_solved = 0 AND is_skipped = 0
        LIMIT 1;",
        ) {
            Ok(val) => val,
            Err(_) => return Err(CustomErrors::StatementFailed),
        };

        match stmt.query_row([], |row| row.get(0)) {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(_) => Err(CustomErrors::Others(
                "Error: While fetching unsolved link".to_owned(),
            )),
        }
    }

    fn mark_as_complete(&self, link: &str) -> Result<(), CustomErrors> {
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

    fn skip_link(&self, link: &str) -> Result<(), CustomErrors> {
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
fn create_db_connection() -> Result<Connection, CustomErrors> {
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

fn show_options(db: Db) -> Result<(), CustomErrors> {
    let options = vec!["Get Link", "Add Link", "Exit"];

    let user_option = match Select::new("select your option", options).prompt() {
        Ok(val) => val,
        Err(e) => match e {
            inquire::InquireError::OperationCanceled => {
                return Err(CustomErrors::OperationCanceled)
            }
            inquire::InquireError::OperationInterrupted => {
                return Err(CustomErrors::OperationInterrupted)
            }
            _ => {
                return Err(CustomErrors::Others(
                    "Error: Unable to show the select menu".to_owned(),
                ))
            }
        },
    };

    let selected_item = match user_option {
        "Get Link" => UserOptions::GetLink,
        "Add Link" => UserOptions::AddLink,
        "Exit" => UserOptions::Exit,
        _ => unreachable!(),
    };

    match selected_item {
        UserOptions::GetLink => user_get_link(&db)?,
        UserOptions::AddLink => user_link_input(&db)?,
        UserOptions::Exit => return Err(CustomErrors::Exit),
    }

    Ok(())
}

fn user_single_link_options(db: &Db, link: &str) -> Result<(), CustomErrors> {
    let options = vec!["Mark As Complete?", "Skip And Get Another Link?"];
    let choice = match Select::new("Select your option", options).prompt() {
        Ok(val) => val,
        Err(_) => {
            return Err(CustomErrors::Others(
                "Error: Something went wrong while showing options".to_owned(),
            ))
        }
    };

    let selected_option = match choice {
        "Mark As Complete?" => InnerOptions::MarkAsComplete,
        "Skip And Get Another Link?" => InnerOptions::SkipAndNext,
        _ => unreachable!(),
    };

    match selected_option {
        InnerOptions::MarkAsComplete => db.mark_as_complete(link)?,
        InnerOptions::SkipAndNext => db.skip_link(link)?,
    };

    Ok(())
}

fn user_get_link(db: &Db) -> Result<(), CustomErrors> {
    let link = match db.get_single_link() {
        Ok(val) => match val {
            Some(link) => {
                println!("Your Link: {}", &link);
                link
            }
            None => {
                println!("No unsolved links, add new links or reset the link status");
                "".to_string()
            }
        },
        Err(e) => return Err(e),
    };

    user_single_link_options(db, &link)?;

    Ok(())
}

fn user_link_input(db: &Db) -> Result<(), CustomErrors> {
    let links = db.get_links()?;
    let validator = move |input: &str| {
        if links.contains(&input.to_owned()) {
            Ok(Validation::Invalid(
                "Duplicate link, enter another link".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    };

    let link = match Text::new("Enter the link:")
        .with_validator(required!())
        .with_validator(validator)
        .with_help_message("enter the link you want to save and hit enter")
        .prompt()
    {
        Ok(val) => val,
        Err(e) => match e {
            inquire::InquireError::OperationCanceled => {
                return Err(CustomErrors::OperationCanceled)
            }
            inquire::InquireError::OperationInterrupted => {
                return Err(CustomErrors::OperationInterrupted)
            }
            _ => {
                return Err(CustomErrors::Others(
                    "Error: Something went wrong while taking link input".to_owned(),
                ))
            }
        },
    };

    _ = db.add_link(link);

    Ok(())
}

fn main() {
    let _conn: Connection = match create_db_connection() {
        Ok(value) => value,
        Err(e) => match e {
            CustomErrors::CacheDirectoryNotFound => {
                println!("Error: The cache directory was not found");
                return;
            }
            CustomErrors::CreateDirectoryFailed => {
                println!("Error: Couldn't create the db directory");
                return;
            }
            CustomErrors::FileCreationFailed(msg) => {
                println!("Error: File creation failed due to:{}", msg);
                return;
            }
            CustomErrors::DBConnectionFailed => {
                println!("Error: DB connection failed");
                return;
            }
            CustomErrors::DBQueryFailed => {
                println!("Error: DB query failed");
                return;
            }
            _ => unreachable!(),
        },
    };

    let db = Db::new(_conn);

    match show_options(db) {
        Ok(_) => (),
        Err(e) => match e {
            CustomErrors::DuplicateLinkValue => {
                println!("Error: Link already exists, input other link")
            }
            CustomErrors::StatementFailed => println!("Error: Failed to execute the statement"),
            CustomErrors::InvalidColumnName(column_name) => {
                println!("Error: column {} does not exist", column_name)
            }
            CustomErrors::OperationCanceled => {
                println!("Error: User cancelled the operation")
            }
            CustomErrors::OperationInterrupted => {
                println!("Error: User forcefully quit the operation")
            }
            CustomErrors::Others(msg) => println!("Error: {}", msg),
            CustomErrors::Exit => println!("You've sucessfully quit the application :)"),
            _ => unreachable!(),
        },
    }
}

