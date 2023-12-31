use crate::print::{pretty_print, pretty_status};
use crate::utility::show_red;
use crate::CustomErrors;
use crate::{database::Db, utility::show_green};
use inquire::{required, validator::Validation, Select, Text};

enum MainMenuOptions {
    Status,
    GetLink,
    AddLink,
    DeleteLink,
    SearchLink,
    Other,
    Exit,
}

enum DeleteOptions {
    DeleteLink,
    MainMenu,
    Exit,
}

enum GetLinkOptions {
    MarkAsComplete,
    Skip,
    MainMenu,
    Exit,
}

enum OtherOptions {
    ShowAllLinks,
    ShowCompletedLinks,
    ShowSkippedLinks,
    SkippedToIncomplete,
    CompletedToIncomplete,
    MainMenu,
    Exit,
}

pub fn show_options(db: &Db) -> Result<(), CustomErrors> {
    let options = vec![
        "Check Status",
        "Get Link",
        "Add Link",
        "Delete Link",
        "Search Link",
        "Other",
        "Exit",
    ];

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
        "Check Status" => MainMenuOptions::Status,
        "Get Link" => MainMenuOptions::GetLink,
        "Add Link" => MainMenuOptions::AddLink,
        "Delete Link" => MainMenuOptions::DeleteLink,
        "Search Link" => MainMenuOptions::SearchLink,
        "Other" => MainMenuOptions::Other,
        "Exit" => MainMenuOptions::Exit,
        _ => unreachable!(),
    };

    match selected_item {
        MainMenuOptions::Status => get_status(db)?,
        MainMenuOptions::GetLink => get_link_options(db)?,
        MainMenuOptions::AddLink => add_link_options(db)?,
        MainMenuOptions::DeleteLink => delete_link_options(db)?,
        MainMenuOptions::SearchLink => search_link_options(db)?,
        MainMenuOptions::Other => show_other_options(db)?,
        MainMenuOptions::Exit => return Err(CustomErrors::Exit),
    }

    Ok(())
}

fn get_status(db: &Db) -> Result<(), CustomErrors> {
    match db.get_status() {
        Ok(val) => {
            match val {
                Some((total_links, completed_links, skipped_links)) => {
                    pretty_status(total_links, completed_links, skipped_links)
                }
                None => pretty_status(0, 0, 0),
            };
        }
        Err(e) => return Err(e),
    };
    Ok(())
}

fn get_link_options(db: &Db) -> Result<(), CustomErrors> {
    let link = match db.get_single_link() {
        Ok(val) => match val {
            Some((link, solved_count)) => {
                pretty_print(&[(link.clone(), solved_count)]);
                link
            }
            None => {
                show_red("No unsolved links, add new links or reset the link status");
                return Ok(());
            }
        },
        Err(e) => return Err(e),
    };

    single_link_options(db, &link)?;

    Ok(())
}

fn add_link_options(db: &Db) -> Result<(), CustomErrors> {
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

    match db.add_link(link.clone()) {
        Ok(_) => show_green(format!("Successfully added the link: {}", link).as_str()),
        Err(e) => return Err(e),
    };

    Ok(())
}

fn delete_link_options(db: &Db) -> Result<(), CustomErrors> {
    let links = db.get_links()?;

    let link = match Select::new("select link to delete", links).prompt() {
        Ok(val) => val,
        Err(_) => {
            return Err(CustomErrors::Others(
                "Error: Something went wrong while deleting links".to_owned(),
            ))
        }
    };

    let options = vec!["Delete Link", "Main Menu", "Exit"];

    let choice = match Select::new("select your option", options).prompt() {
        Ok(val) => val,
        Err(_) => {
            return Err(CustomErrors::Others(
                "Error: Something went wrong while showing delete options".to_owned(),
            ))
        }
    };

    let selected_option = match choice {
        "Delete Link" => DeleteOptions::DeleteLink,
        "Main Menu" => DeleteOptions::MainMenu,
        "Exit" => DeleteOptions::Exit,
        _ => unreachable!(),
    };

    match selected_option {
        DeleteOptions::DeleteLink => {
            match db.delete_link(link) {
                Ok(_) => show_green("Successfully deleted the link"),
                Err(e) => return Err(e),
            };
        }
        DeleteOptions::MainMenu => (),
        DeleteOptions::Exit => return Err(CustomErrors::Exit),
    };

    Ok(())
}

fn single_link_options(db: &Db, link: &str) -> Result<(), CustomErrors> {
    let options = vec![
        "Mark As Complete?",
        "Skip And Go To Main Menu?",
        "Main Menu",
        "Exit",
    ];
    let choice = match Select::new("Select your option", options).prompt() {
        Ok(val) => val,
        Err(_) => {
            return Err(CustomErrors::Others(
                "Error: Something went wrong while showing options".to_owned(),
            ))
        }
    };

    let selected_option = match choice {
        "Mark As Complete?" => GetLinkOptions::MarkAsComplete,
        "Skip And Go To Main Menu?" => GetLinkOptions::Skip,
        "Main Menu" => GetLinkOptions::MainMenu,
        "Exit" => GetLinkOptions::Exit,
        _ => unreachable!(),
    };

    match selected_option {
        GetLinkOptions::MarkAsComplete => {
            match db.mark_as_complete(link) {
                Ok(_) => show_green("Successfully marked the link as completed"),
                Err(e) => return Err(e),
            };
        }
        GetLinkOptions::Skip => {
            match db.skip_link(link) {
                Ok(_) => show_green("Successfully skipped the link"),
                Err(e) => return Err(e),
            };
        }
        GetLinkOptions::MainMenu => (),
        GetLinkOptions::Exit => return Err(CustomErrors::Exit),
    };

    Ok(())
}

fn search_link_options(db: &Db) -> Result<(), CustomErrors> {
    let links = db.get_links()?;

    let link = match Select::new("select link or type keywords", links).prompt() {
        Ok(val) => val,
        Err(_) => {
            return Err(CustomErrors::Others(
                "Error: Something went wrong while searching links".to_owned(),
            ))
        }
    };

    match db.get_searched_link_count(&link) {
        Ok(val) => match val {
            Some(solved_count) => pretty_print(&[(link.clone(), solved_count)]),
            None => pretty_print(&[(link.clone(), 0)]),
        },
        Err(e) => return Err(e),
    };

    single_link_options(db, &link)?;

    Ok(())
}

fn show_other_options(db: &Db) -> Result<(), CustomErrors> {
    let options = vec![
        "Show All Links?",
        "Show Completed Links?",
        "Show Skipped Links?",
        "Change All Skipped Links to Incomplete?",
        "Change All Completed Links to Incomplete?",
        "Main Menu",
        "Exit",
    ];
    let choice = match Select::new("Select your option", options).prompt() {
        Ok(val) => val,
        Err(_) => {
            return Err(CustomErrors::Others(
                "Error: Something went wrong while showing options".to_owned(),
            ))
        }
    };

    let selected_option = match choice {
        "Show All Links?" => OtherOptions::ShowAllLinks,
        "Show Completed Links?" => OtherOptions::ShowCompletedLinks,
        "Show Skipped Links?" => OtherOptions::ShowSkippedLinks,
        "Change All Skipped Links to Incomplete?" => OtherOptions::SkippedToIncomplete,
        "Change All Completed Links to Incomplete?" => OtherOptions::CompletedToIncomplete,
        "Main Menu" => OtherOptions::MainMenu,
        "Exit" => OtherOptions::Exit,
        _ => unreachable!(),
    };

    match selected_option {
        OtherOptions::ShowAllLinks => match db.get_all_links() {
            Ok(val) => match val {
                Some(all_links) => pretty_print(&all_links),
                None => show_red("No Links present in the database :("),
            },
            Err(e) => return Err(e),
        },
        OtherOptions::ShowCompletedLinks => match db.get_completed_links() {
            Ok(val) => match val {
                Some(completed_links) => pretty_print(&completed_links),
                None => show_red("No Completed Links :("),
            },
            Err(e) => return Err(e),
        },
        OtherOptions::ShowSkippedLinks => match db.get_skipped_links() {
            Ok(val) => match val {
                Some(skipped_links) => pretty_print(&skipped_links),
                None => show_red("No Skipped Links :)"),
            },
            Err(e) => return Err(e),
        },
        OtherOptions::SkippedToIncomplete => {
            match db.skipped_to_incomplete() {
                Ok(count) => show_green(
                    format!("Changed {} Skipped Links To Incomplete Links", count).as_str(),
                ),
                Err(e) => return Err(e),
            };
        }
        OtherOptions::CompletedToIncomplete => {
            match db.completed_to_incomplete() {
                Ok(count) => show_green(
                    format!("Changed {} Completed Links To Incomplete Links", count).as_str(),
                ),
                Err(e) => return Err(e),
            };
        }
        OtherOptions::MainMenu => (),
        OtherOptions::Exit => return Err(CustomErrors::Exit),
    }

    Ok(())
}
