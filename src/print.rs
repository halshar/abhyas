use tabled::settings::Style;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Links {
    id: usize,
    link: String,
    solved_count: i32,
}

pub fn pretty_print(data: &[(String, i32)]) {
    let new_data: Vec<Links> = data
        .iter()
        .enumerate()
        .map(|(id, (link, solved_count))| Links {
            id: id + 1,
            link: link.to_string(),
            solved_count: *solved_count,
        })
        .collect();

    let mut table = Table::new(new_data);
    let table = table.with(Style::modern());
    let table_string = table.to_string();

    println!("{}", table_string);
}
