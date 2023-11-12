use std::fs;
use std::path::Path;

fn create_file() -> std::io::Result<()> {
    // create cache directory if not present
    let cache_dir = dirs::cache_dir().unwrap();
    let dir_name = &cache_dir.join("abhyas");
    if !Path::new(&dir_name).exists() {
        fs::create_dir(&dir_name)?;
    }

    // create db file if not present
    let file_name = &dir_name.join("abhyas.db");
    if !Path::new(&file_name).exists() {
        fs::File::create(&file_name)?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    create_file()
}
