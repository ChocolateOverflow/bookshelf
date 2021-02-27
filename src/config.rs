use serde_yaml::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    YamlError(serde_yaml::Error),
}

pub struct Config {
    pub index_file: PathBuf,
    pub modules_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Config {
    pub fn default() -> Config {
        let config_root: PathBuf;
        if let Some(mut dir) = dirs_next::config_dir() {
            dir.push("bookshelf");
            config_root = dir;
        } else {
            config_root = PathBuf::from("/bookshelf");
        }
        let mut index_file = config_root.clone();
        index_file.push("data/index");
        let mut modules_dir = config_root.clone();
        modules_dir.push("modules");
        let mut data_dir = config_root.clone();
        data_dir.push("data");
        Config {
            index_file,
            modules_dir,
            data_dir,
        }
    }

    pub fn update(&mut self, config_file: &PathBuf) -> Result<(), ConfigError> {
        match File::open(config_file) {
            Ok(file) => {
                let r: Result<HashMap<String, String>, serde_yaml::Error> = from_reader(file);
                match r {
                    Ok(data) => {
                        if let Some(index_file) = data.get("index_file") {
                            match shellexpand::full(index_file) {
                                Ok(index_file) => {
                                    self.index_file = PathBuf::from(index_file.into_owned());
                                }
                                Err(e) => println!("Error expanding path: {}", e),
                            }
                        }
                        if let Some(modules_dir) = data.get("modules_dir") {
                            match shellexpand::full(modules_dir) {
                                Ok(modules_dir) => {
                                    self.modules_dir = PathBuf::from(modules_dir.into_owned());
                                }
                                Err(e) => println!("Error expanding path: {}", e),
                            }
                        }
                        if let Some(data_dir) = data.get("data_dir") {
                            match shellexpand::full(data_dir) {
                                Ok(data_dir) => {
                                    self.data_dir = PathBuf::from(data_dir.into_owned());
                                }
                                Err(e) => println!("Error expanding path: {}", e),
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(ConfigError::YamlError(e)),
                }
            }
            Err(e) => Err(ConfigError::IoError(e)),
        }
    }
}
