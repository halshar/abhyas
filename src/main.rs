use rusqlite::Connection;

#[derive(Debug)]
pub enum CustomErrors {
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
    WriteFailed(String),
    Exit,
}

mod cli;
mod database;
mod print;
mod utility;

use cli::show_options;
use database::Db;
use utility::{create_db_connection, show_green, show_red};

fn main() {
    let _conn: Connection = match create_db_connection() {
        Ok(value) => value,
        Err(e) => match e {
            CustomErrors::CacheDirectoryNotFound => {
                return show_red("Error: The cache directory was not found");
            }
            CustomErrors::CreateDirectoryFailed => {
                return show_red("Error: Couldn't create the db directory");
            }
            CustomErrors::FileCreationFailed(msg) => {
                return show_red(format!("Error: File creation failed due to:{}", msg).as_str());
            }
            CustomErrors::DBConnectionFailed => {
                return show_red(format!("Error: DB connection failed").as_str());
            }
            CustomErrors::DBQueryFailed => {
                return show_red(format!("Error: DB query failed").as_str())
            }
            _ => unreachable!(),
        },
    };

    let db = Db::new(_conn);

    loop {
        match show_options(&db) {
            Ok(_) => (),
            Err(e) => match e {
                CustomErrors::DuplicateLinkValue => {
                    return show_red("Error: Link already exists, input other link");
                }
                CustomErrors::StatementFailed => {
                    return show_red("Error: Failed to execute the statement");
                }
                CustomErrors::InvalidColumnName(column_name) => {
                    return show_red(
                        format!("Error: column {} does not exist", column_name).as_str(),
                    );
                }
                CustomErrors::OperationCanceled => {
                    return show_red("Error: User cancelled the operation");
                }
                CustomErrors::OperationInterrupted => {
                    return show_red("Error: User forcefully quit the operation");
                }
                CustomErrors::Others(msg) => {
                    return show_red(format!("Error: {}", msg).as_str());
                }
                CustomErrors::WriteFailed(msg) => {
                    return show_red(format!("Error: {}", msg).as_str());
                }
                CustomErrors::Exit => {
                    return show_green("You've successfully quit the application :)");
                }
                _ => unreachable!(),
            },
        }
    }
}

