use bincode::{deserialize, serialize, ErrorKind};
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::shelf::*;

/// Create necessary directories if not already present.
pub fn create_dirs(data_dir: &PathBuf, modules_dir: &PathBuf) {
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).expect("Error: unable to create data directory");
    }
    if !modules_dir.exists() {
        std::fs::create_dir_all(modules_dir).expect("Error: unable to create modules directory");
    }
}

/// Read the index file and returns a Shelf object.
pub fn load_shelf(file: &PathBuf) -> Shelf {
    if Path::new(&file).exists() {
        let data: Vec<u8> = std::fs::read(file).expect("Unable to read index file");
        let data: Result<Shelf, Box<ErrorKind>> = deserialize(&data);
        match data {
            Ok(d) => d,
            Err(_e) => Shelf::new(),
        }
    } else {
        Shelf::new()
    }
}

/// Write Shelf object to index file
pub fn save_shelf(shelf: &Shelf, index_file: &PathBuf) {
    let data = serialize(&shelf);
    match data {
        Ok(d) => {
            std::fs::write(index_file, d).expect("Unable to write to index file");
        }
        Err(e) => {
            println!("Error writing index file: {}", e)
        }
    };
}

pub fn import_shelf(shelf: &mut Shelf, index_file: &PathBuf) {
    match File::open(&index_file) {
        Ok(f) => {
            let data: Result<Shelf, serde_yaml::Error> = serde_yaml::from_reader(f);
            match data {
                Ok(new_shelf) => {
                    shelf.import(&new_shelf);
                }
                Err(e) => println!("Error reading yaml index file: {}", e),
            }
        }
        Err(e) => {
            println!("Error openining yaml index file: {}", e);
        }
    }
}

pub fn export_shelf(shelf: &Shelf, index_file: &PathBuf) {
    let data = serde_yaml::to_string(&shelf).expect("Failed to export shelf");
    match std::fs::write(index_file, data){
        Ok(()) => println!("Successfully exported index"),
        Err(e) => println!("Error exporting index: {}", e)
    }
}
