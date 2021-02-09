use bincode::{deserialize, serialize, ErrorKind};
use shellexpand;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::shelf::*;

pub fn load_config(config_file: &PathBuf) -> HashMap<String, String> {
    let mut config: HashMap<String, String> = HashMap::new();

    // Defaults
    let path: &str = "~/.config/bookshelf";
    config.insert(
        "config_file".to_string(),
        format!("{}/{}", path, "bookshelf.yaml"),
    );
    config.insert(
        "index_file".to_string(),
        format!("{}/{}", path, "data/index"),
    );
    config.insert("modules_dir".to_string(), format!("{}/{}", path, "modules"));
    config.insert("data_dir".to_string(), format!("{}/{}", path, "data"));

    // Read from config file
    match File::open(&config_file) {
        Ok(f) => {
            let data: Result<HashMap<String, String>, serde_yaml::Error> =
                serde_yaml::from_reader(f);
            match data {
                Ok(data) => config.extend(data),
                Err(e) => println!("Error reading config file: {}", e),
            }
        }
        Err(e) => {
            println!("Error openining config file (try creating ~/.config/bookshelf/bookshelf.yaml or run with --config)");
        }
    }

    // expand path
    let p = shellexpand::full(config.get("modules_dir").unwrap());
    match p {
        Ok(p) => {
            config.insert("modules_dir".to_string(), p.to_string());
        }
        Err(e) => {
            println!("Error getting modules directory: {}", e);
        }
    };
    let p = shellexpand::full(config.get("data_dir").unwrap());
    match p {
        Ok(p) => {
            config.insert("data_dir".to_string(), p.to_string());
        }
        Err(e) => {
            println!("Error getting data directory: {}", e);
        }
    };
    let p = shellexpand::full(config.get("index_file").unwrap());
    match p {
        Ok(p) => {
            config.insert("index_file".to_string(), p.to_string());
        }
        Err(e) => {
            println!("Error getting index_file: {}", e);
        }
    };

    return config;
}

pub fn create_dirs(data_dir: &PathBuf, modules_dir: &PathBuf) {
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).expect("Error: unable to create data directory");
    }
    if !modules_dir.exists() {
        std::fs::create_dir_all(modules_dir).expect("Error: unable to create modules directory");
    }
}

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
