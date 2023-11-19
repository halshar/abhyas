pub enum CustomErrors {
    InsufficientArgs,
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

use utility::{run, show_green, show_red};

fn main() {
    if let Err(e) = run() {
        match e {
            CustomErrors::InsufficientArgs => show_red("Usage:\nabhyas = Normal binary execution\nabhyas --file <file_name> = To add links from file"),
            CustomErrors::CacheDirectoryNotFound => show_red("Error: The cache directory was not found"),
            CustomErrors::CreateDirectoryFailed => show_red("Error: Couldn't create the db directory"),
            CustomErrors::FileCreationFailed(msg) => show_red(&format!("Error: File creation failed due to: {}", msg)),
            CustomErrors::DBConnectionFailed => show_red("Error: DB connection failed"),
            CustomErrors::DBQueryFailed => show_red("Error: DB query failed"),
            CustomErrors::DuplicateLinkValue => show_red("Error: Link already exists, input other link"),
            CustomErrors::StatementFailed => show_red("Error: Failed to execute the statement"),
            CustomErrors::InvalidColumnName(column_name) => show_red(&format!("Error: column {} does not exist", column_name)),
            CustomErrors::OperationCanceled => show_red("Error: User cancelled the operation"),
            CustomErrors::OperationInterrupted => show_red("Error: User forcefully quit the operation"),
            CustomErrors::Others(msg) => show_red(&format!("Error: {}", msg)),
            CustomErrors::WriteFailed(msg) => show_red(&format!("Error: {}", msg)),
            CustomErrors::Exit => show_green("You've successfully quit the application :)"),
        };
    }
}
