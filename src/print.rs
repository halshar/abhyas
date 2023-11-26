use tabled::{
    settings::{object::Segment, Alignment, Modify, Style},
    Table, Tabled,
};

#[derive(Tabled)]
struct Links {
    id: usize,
    link: String,
    solved_count: i32,
}

#[derive(Tabled)]
struct Status {
    total_links: i32,
    completed_links: i32,
    skipped_links: i32,
}

pub fn pretty_status(total_links: i32, completed_links: i32, skipped_links: i32) {
    let data = vec![Status {
        total_links,
        completed_links,
        skipped_links,
    }];

    let mut table = Table::new(data);
    table.with(
        Modify::new(Segment::all())
            .with(Alignment::center())
            .with(Alignment::top()),
    );

    let table = table.with(Style::modern());
    let table_string = table.to_string();

    println!("{}", table_string);
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
