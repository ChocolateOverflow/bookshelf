use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Command,
};

#[derive(Debug)]
struct Module {
    mod_file: PathBuf,
    media_type: String,
}

impl Module {
    pub fn new(mod_file: PathBuf, media_type: String) -> Module {
        Module {
            mod_file,
            media_type,
        }
    }

    fn is_url_valid(&self, url: &str) -> bool {
        match Command::new(&self.mod_file).args(&["check", url]).output() {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => {
                    if strip(out) == "1" {
                        return true;
                    } else {
                        return false;
                    }
                }
                Err(e) => {
                    println!("Error validating URL: {}", e);
                    return false;
                }
            },
            Err(e) => {
                println!("Error validating URL: {}", e);
                return false;
            }
        }
    }

    fn derive_code(&self, url: &str) -> Option<String> {
        match Command::new(&self.mod_file).args(&["code", url]).output() {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => Some(out),
                Err(e) => {
                    println!("Error deriving code: {}", e);
                    return None;
                }
            },
            Err(e) => {
                println!("Error deriving code: {}", e);
                None
            }
        }
    }

    fn derive_url(&self, code: &str) -> Option<String> {
        match Command::new(&self.mod_file).args(&["url", code]).output() {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => Some(out),
                Err(e) => {
                    println!("Error deriving URL: {}", e);
                    return None;
                }
            },
            Err(e) => {
                println!("Error deriving URL: {}", e);
                None
            }
        }
    }

    fn get_metadata(&self, code: &str) -> Option<(String, String, String)> {
        match Command::new(&self.mod_file)
            .args(&["metadata", code])
            .output()
        {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => {
                    let out = strip(out);
                    let mut lines = out.lines();
                    let title = lines.next();
                    let authors = lines.next();
                    let tags = lines.next();
                    return Some((title?.to_string(), authors?.to_string(), tags?.to_string()));
                }
                Err(e) => {
                    println!("Error deriving URL: {}", e);
                    return None;
                }
            },
            Err(e) => {
                println!("Error deriving URL: {}", e);
                None
            }
        }
    }

    fn download(&self, code: &str, dest_dir: &str) {
        Command::new(&self.mod_file).args(&["download", code, dest_dir]);
    }
}

pub struct ModuleHandler {
    modules: HashMap<String, Module>,
}

impl ModuleHandler {
    pub fn new(modules_dir: &PathBuf) -> ModuleHandler {
        let mut modules: HashMap<String, Module> = HashMap::new();
        match std::fs::read_dir(&modules_dir) {
            Ok(files) => {
                // Look at all files in `modules_dir/`
                for file in files {
                    match file {
                        Ok(file) => {
                            let pb: PathBuf = file.path();
                            // if execution of `module media` is successful
                            match Command::new(&pb).args(&["media"]).output() {
                                Ok(out) => {
                                    // Get media type
                                    let media: Option<String> = {
                                        match String::from_utf8(out.stdout) {
                                            Ok(media) => Some(media),
                                            Err(_e) => None,
                                        }
                                    };
                                    // Get module name ( = fie name)
                                    let name: Option<&str> =
                                        pb.as_path().file_name().and_then(std::ffi::OsStr::to_str);
                                    // Add module if got both media type and name
                                    if let (Some(name), Some(media)) = (name, media) {
                                        modules.insert(name.to_string(), Module::new(pb, media));
                                    }
                                }
                                Err(_e) => {}
                            }
                        }
                        Err(e) => println!("Error getting module files: {:?}", e),
                    }
                }
            }
            Err(e) => {
                println!("Error reading modules directory: {:?}", e);
            }
        };
        ModuleHandler { modules }
    }

    pub fn derive_module(&self, url: &str) -> Option<&str> {
        for (name, module) in self.modules.iter() {
            if module.is_url_valid(url) {
                return Some(name);
            }
        }
        return None;
    }

    pub fn derive_code(&self, module: &str, url: &str) -> Option<String> {
        match self.modules.get(module) {
            Some(module) => module.derive_code(url),
            None => None,
        }
    }

    pub fn derive_url(&self, module: &str, code: &str) -> Option<String> {
        match self.modules.get(module) {
            Some(module) => module.derive_url(code),
            None => None,
        }
    }

    pub fn get_media_type(&self, module: &str) -> Option<String> {
        match self.modules.get(module) {
            Some(m) => Some(m.media_type.clone()),
            None => None,
        }
    }

    pub fn list_modules(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        for key in self.modules.keys() {
            result.insert(key.clone());
        }
        result
    }

    pub fn has_module(&self, module: &str) -> bool {
        self.modules.contains_key(module)
    }

    pub fn get_metadata(&self, module: &str, code: &str) -> Option<(String, String, String)> {
        match self.modules.get(module) {
            Some(module) => module.get_metadata(code),
            None => None,
        }
    }

    pub fn download(&self, module: &str, code: &str, dest_dir: &String) {
        match self.modules.get(module) {
            Some(m) => m.download(code, &dest_dir[..]),
            None => {
                println!("Module '{}' not found", &module)
            }
        }
    }
}

fn strip(mut string: String) -> String {
    while string.ends_with("\n") || string.ends_with(" ") {
        string.pop();
    }
    return string;
}
