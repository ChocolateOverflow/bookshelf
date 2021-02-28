use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    process::Command,
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum ModuleError {
    FromUtf8Error(FromUtf8Error),
    IoError(std::io::Error),
    NoValidModule,
    MalformedMetadata,
}

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
    fn is_url_valid(&self, url: &str) -> Result<bool, ModuleError> {
        match Command::new(&self.mod_file).args(&["check", url]).output() {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => {
                    if strip(out) == "1" {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                Err(e) => Err(ModuleError::FromUtf8Error(e)),
            },
            Err(e) => Err(ModuleError::IoError(e)),
        }
    }

    /// Given a URL, return the respective item code
    fn derive_code(&self, url: &str) -> Result<String, ModuleError> {
        match Command::new(&self.mod_file).args(&["code", url]).output() {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => Ok(out),
                Err(e) => Err(ModuleError::FromUtf8Error(e)),
            },
            Err(e) => Err(ModuleError::IoError(e)),
        }
    }

    /// Given an item code, return the respective URL
    fn derive_url(&self, code: &str) -> Result<String, ModuleError> {
        match Command::new(&self.mod_file).args(&["url", code]).output() {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => Ok(out),
                Err(e) => Err(ModuleError::FromUtf8Error(e)),
            },
            Err(e) => Err(ModuleError::IoError(e)),
        }
    }

    /// Get the title, authors, and genres of a book
    fn get_metadata(&self, code: &str) -> Result<(String, String, String), ModuleError> {
        match Command::new(&self.mod_file)
            .args(&["metadata", code])
            .output()
        {
            Ok(out) => match String::from_utf8(out.stdout) {
                Ok(out) => {
                    let out = strip(out);
                    let mut lines = out.lines();
                    if let Some(title) = lines.next() {
                        if let Some(authors) = lines.next() {
                            if let Some(genres) = lines.next() {
                                return Ok((
                                    title.to_string(),
                                    authors.to_string(),
                                    genres.to_string(),
                                ));
                            } else {
                                Err(ModuleError::MalformedMetadata)
                            }
                        } else {
                            Err(ModuleError::MalformedMetadata)
                        }
                    } else {
                        Err(ModuleError::MalformedMetadata)
                    }
                }
                Err(e) => Err(ModuleError::FromUtf8Error(e)),
            },
            Err(e) => Err(ModuleError::IoError(e)),
        }
    }

    /// Download a book given its code. Everything here is handled by the module.
    fn download(&self, code: &str, dest_dir: &str) {
        let pb = PathBuf::from(&dest_dir);
        if !&pb.exists() {
            match std::fs::create_dir_all(pb) {
                Ok(()) => {
                    match Command::new(&self.mod_file).args(&["download", code, dest_dir]).output(){
                        Ok(output) => {
                            let out = String::from_utf8(output.stdout).expect("Error converting UTF8 output");
                            println!("{}", out);
                        }
                        Err(e) => println!("Error downloading item: {}", e)
                    }
                }
                Err(e) => {
                    println!("Error creating item data directory: {}", e);
                }
            }
        }
    }
}

/// Handles everything about modules.
pub struct ModuleHandler {
    modules: BTreeMap<String, Module>,
}

impl ModuleHandler {
    /// Load  available modules from modules_path
    pub fn new(modules_dir: &PathBuf) -> ModuleHandler {
        let mut modules: BTreeMap<String, Module> = BTreeMap::new();
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
                        Err(_e) => {}
                    }
                }
            }
            Err(_e) => {}
        };
        ModuleHandler { modules }
    }

    /// Derives the appropriate module given a URL
    pub fn derive_module(&self, url: &str) -> Result<&String, ModuleError> {
        for (name, module) in self.modules.iter() {
            match module.is_url_valid(url) {
                Ok(is_valid) => {
                    if is_valid {
                        return Ok(name);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(ModuleError::NoValidModule)
    }

    /// Given a module and URL, derive the corresponding code
    pub fn derive_code(&self, module: &str, url: &str) -> Result<String, ModuleError> {
        match self.modules.get(module) {
            Some(module) => module.derive_code(url),
            None => Err(ModuleError::NoValidModule),
        }
    }

    /// Given a module and code, derive the corresponding URL
    pub fn derive_url(&self, module: &str, code: &str) -> Result<String, ModuleError> {
        match self.modules.get(module) {
            Some(module) => module.derive_url(code),
            None => Err(ModuleError::NoValidModule),
        }
    }

    /// Get the media type handled by a module
    pub fn get_media_type(&self, module: &str) -> Result<String, ModuleError> {
        match self.modules.get(module) {
            Some(m) => Ok(m.media_type.clone()),
            None => Err(ModuleError::NoValidModule),
        }
    }

    /// Get a set of available modules
    pub fn list_modules(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        for key in self.modules.keys() {
            result.insert(key.clone());
        }
        result
    }

    /// Check if a module is available
    pub fn has_module(&self, module: &str) -> bool {
        self.modules.contains_key(module)
    }

    /// Given a module and code, get the title, authors, and genres of the corresponding item
    pub fn get_metadata(
        &self,
        module: &str,
        code: &str,
    ) -> Result<(String, String, String), ModuleError> {
        match self.modules.get(module) {
            Some(module) => module.get_metadata(code),
            None => Err(ModuleError::NoValidModule),
        }
    }

    /// Given a module and code, download item to the provided directory
    pub fn download(&self, module: &str, code: &str, dest_dir: &PathBuf) -> Result<(), ModuleError> {
        let dest_dir = &*dest_dir.clone().into_os_string().into_string().unwrap();
        match self.modules.get(module) {
            Some(m) => {
                m.download(code, dest_dir);
                Ok(())
            }
            None => Err(ModuleError::NoValidModule),
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
