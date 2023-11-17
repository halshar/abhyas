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
    Exit,
}

mod cli;
mod database;
mod utility;

use cli::show_options;
use database::Db;
use utility::create_db_connection;

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

    loop {
        match show_options(&db) {
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
                    println!("Error: User cancelled the operation");
                    break;
                }
                CustomErrors::OperationInterrupted => {
                    println!("Error: User forcefully quit the operation");
                    break;
                }
                CustomErrors::Others(msg) => println!("Error: {}", msg),
                CustomErrors::Exit => {
                    println!("You've sucessfully quit the application :)");
                    break;
                }
                _ => unreachable!(),
            },
        }
    }
}

