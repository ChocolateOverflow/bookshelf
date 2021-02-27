use std::path::PathBuf;

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

    pub fn update(&mut self, config_file: &PathBuf) {}
}
