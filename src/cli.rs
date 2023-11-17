use crate::utility::show_red;
use crate::CustomErrors;
use crate::{database::Db, utility::show_green};
use inquire::{required, validator::Validation, Select, Text};

enum MainMenuOptions {
    GetLink,
    AddLink,
    Exit,
}

enum GetLinkOptions {
    MarkAsComplete,
    Skip,
    MainMenu,
    Exit,
}

pub fn show_options(db: &Db) -> Result<(), CustomErrors> {
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
        "Get Link" => MainMenuOptions::GetLink,
        "Add Link" => MainMenuOptions::AddLink,
        "Exit" => MainMenuOptions::Exit,
        _ => unreachable!(),
    };

    match selected_item {
        MainMenuOptions::GetLink => get_link_options(&db)?,
        MainMenuOptions::AddLink => add_link_options(&db)?,
        MainMenuOptions::Exit => return Err(CustomErrors::Exit),
    }

    Ok(())
}

fn get_link_options(db: &Db) -> Result<(), CustomErrors> {
    let link = match db.get_single_link() {
        Ok(val) => match val {
            Some(link) => {
                show_green(format!("Your Link: {}", &link).as_str());
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

    loop {
        match selected_option {
            GetLinkOptions::MarkAsComplete => {
                match db.mark_as_complete(link) {
                    Ok(_) => show_green("Successfully marked the link as completed"),
                    Err(e) => return Err(e),
                };
                break;
            }
            GetLinkOptions::Skip => {
                match db.skip_link(link) {
                    Ok(_) => show_green("Successfully skipped the link"),
                    Err(e) => return Err(e),
                };
                break;
            }
            GetLinkOptions::MainMenu => break,
            GetLinkOptions::Exit => return Err(CustomErrors::Exit),
        };
    }

    Ok(())
}
