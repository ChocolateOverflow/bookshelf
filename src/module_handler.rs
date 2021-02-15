use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Command,
};

/// A module is defined by a name and the path to the module file.
/// The media_type parameter is set when loading modules to speed things up.
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

    /// Check if a URL can be handled by the module
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

    /// Given a URL, return the respective item code
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

    /// Given an item code, return the respective URL
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

    /// Get the title, authors, and tags of a book
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

    /// Download a book given its code. Everything here is handled by the module.
    fn download(&self, code: &str, dest_dir: &str) {
        Command::new(&self.mod_file).args(&["download", code, dest_dir]);
    }
}

/// Handles everything about modules.
pub struct ModuleHandler {
    modules: HashMap<String, Module>,
}

impl ModuleHandler {

    /// Load  available modules from modules_path
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

    /// Derives the appropriate module given a URL
    pub fn derive_module(&self, url: &str) -> Option<&str> {
        for (name, module) in self.modules.iter() {
            if module.is_url_valid(url) {
                return Some(name);
            }
        }
        return None;
    }

    /// Given a module and URL, derive the corresponding code
    pub fn derive_code(&self, module: &str, url: &str) -> Option<String> {
        match self.modules.get(module) {
            Some(module) => module.derive_code(url),
            None => None,
        }
    }

    /// Given a module and code, derive the corresponding URL
    pub fn derive_url(&self, module: &str, code: &str) -> Option<String> {
        match self.modules.get(module) {
            Some(module) => module.derive_url(code),
            None => None,
        }
    }

    /// Get the media type handled by a module
    pub fn get_media_type(&self, module: &str) -> Option<String> {
        match self.modules.get(module) {
            Some(m) => Some(m.media_type.clone()),
            None => None,
        }
    }

    /// Get a set of available modules
    pub fn list_modules(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        for key in self.modules.keys() {
            result.insert(key.clone());
        }
        result
    }

    /// Check if a module is available
    pub fn has_module(&self, module: &str) -> bool {
        self.modules.contains_key(module)
    }

    /// Given a module and code, get the title, authors, and tags of the corresponding item
    pub fn get_metadata(&self, module: &str, code: &str) -> Option<(String, String, String)> {
        match self.modules.get(module) {
            Some(module) => module.get_metadata(code),
            None => None,
        }
    }

    /// Given a module and code, download item to the provided directory
    pub fn download(&self, module: &str, code: &str, dest_dir: &String) {
        match self.modules.get(module) {
            Some(m) => m.download(code, &dest_dir[..]),
            None => {
                println!("Module '{}' not found", &module)
            }
        }
    }
}

/// Strip tailing newlines and spaces from String
fn strip(mut string: String) -> String {
    while string.ends_with("\n") || string.ends_with(" ") {
        string.pop();
    }
    return string;
}
