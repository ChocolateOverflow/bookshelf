use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Items to be stored in the Shelf
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    title: String,
    authors: HashSet<String>,
    tags: HashSet<String>,
}

impl Item {
    pub fn new(title: String, authors: HashSet<String>, tags: HashSet<String>) -> Item {
        Item {
            title,
            authors,
            tags,
        }
    }

    /// Get the title, authors, and tags of the item
    pub fn export(&self) -> (&String, &HashSet<String>, &HashSet<String>) {
        return (&self.title, &self.authors, &self.tags);
    }
}

/// The shelf indexes all items and a list of favorites
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shelf {
    index: HashMap<(String, String), Item>,
    favorites: HashSet<(String, String)>,
}

impl Shelf {
    pub fn new() -> Shelf {
        Shelf {
            index: HashMap::new(),
            favorites: HashSet::new(),
        }
    }

    /// Check it an item is in the shelf
    pub fn has_item(&self, module: &str, code: &str) -> bool {
        self.index
            .contains_key(&(module.to_string(), code.to_string()))
    }

    // Add to index but not download
    pub fn add_item(
        &mut self,
        module: &str,
        code: &str,
        title: String,
        authors: HashSet<String>,
        tags: HashSet<String>,
    ) {
        self.index.insert(
            (module.to_string(), code.to_string()),
            Item::new(title, authors, tags),
        );
    }

    /// Search for items matching the provided parameters
    pub fn search_item(
        &self,
        module: Option<&str>,
        title_regex: Option<&str>,
        authors: Option<&str>,
        tags: Option<&str>,
        blacklist: Option<&str>,
        broad_search: bool,
        favorite: bool,
    ) -> Result<HashSet<(String, String)>, regex::Error> {
        // --favorite
        let mut result: HashSet<(String, String)> = if favorite {
            self.favorites.clone()
        } else {
            self.index.keys().cloned().collect()
        };

        // --modules
        if let Some(module) = module {
            let tmp = result.clone();
            for (m, c) in tmp {
                if m != module {
                    result.remove(&(m, c));
                }
            }
        }

        // --authors (match if any author matches)
        if let Some(authors) = authors {
            for key in result.clone() {
                for author in authors.split(",") {
                    if let Some(item) = self.index.get(&key) {
                        if !item.authors.contains(&author.to_string()) {
                            result.remove(&key);
                        }
                    } else {
                        result.remove(&key);
                    }
                }
            }
        }

        // --tags
        if let Some(tags) = tags {
            if broad_search {
                // broad search (match item if at least 1 tag matches)
                for key in result.clone() {
                    for tag in tags.split(",") {
                        if let Some(item) = self.index.get(&key) {
                            if !item.tags.contains(&tag.to_string()) {
                                result.remove(&key);
                            }
                        } else {
                            result.remove(&key);
                        }
                    }
                }
            } else {
                // normal search (match item if all tags match)
                for key in result.clone() {
                    let mut matches: bool = true;
                    for tag in tags.split(",") {
                        if let Some(item) = self.index.get(&key) {
                            if !item.tags.contains(&tag.to_string()) {
                                matches = false;
                            }
                        } else {
                            result.remove(&key);
                        }
                    }
                    if !matches {
                        result.remove(&key);
                    }
                }
            }
        }

        // --blacklist
        if let Some(blacklist) = blacklist {
            for key in result.clone() {
                for tag in blacklist.split(",") {
                    if let Some(item) = self.index.get(&key) {
                        if item.tags.contains(&tag.to_string()) {
                            result.remove(&key);
                        }
                    } else {
                        result.remove(&key);
                    }
                }
            }
        }

        // --title (match regex against title)
        if let Some(title_regex) = title_regex {
            match Regex::new(title_regex) {
                Ok(regex) => {
                    for key in result.clone() {
                        if let Some(item) = self.index.get(&key) {
                            if !&regex.is_match(&item.title) {
                                result.remove(&key);
                            }
                        } else {
                            result.remove(&key);
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(result)
    }

    /// Remove item from index (and favorites)
    pub fn remove_item(&mut self, module: &str, code: &str) {
        let key: (String, String) = (module.to_string(), code.to_string());
        self.index.remove(&key);
        self.favorites.remove(&key);
    }

    /// Get the item corresponding to the module and code
    pub fn get_item(&self, module: &str, code: &str) -> Option<&Item> {
        self.index.get(&(module.to_string(), code.to_string()))
    }

    /// Edit item with provided parameters
    pub fn edit_item(
        &mut self,
        module: Option<&str>,
        code: Option<&str>,
        title: Option<&str>,
        authors: Option<&str>,
        tags: Option<&str>,
        favorite: bool,
    ) {
        // these 2 are required and can be safely unwrap'd
        let k = (module.unwrap().to_string(), code.unwrap().to_string());

        // update values
        if let Some(item) = self.index.get_mut(&k) {
            if let Some(t) = title {
                item.title = t.to_string();
            }
            if let Some(s) = authors {
                let mut authors: HashSet<String> = HashSet::new();
                for author in s.split(",") {
                    authors.insert(author.to_string());
                }
                item.authors = authors;
            }
            if let Some(t) = tags {
                let mut tags: HashSet<String> = HashSet::new();
                for tag in t.split(",") {
                    tags.insert(tag.to_string());
                }
                item.tags = tags;
            }
        }
        // insert if item wasn't in favorites
        if favorite {
            if !self.favorites.remove(&k) {
                self.favorites.insert(k);
            }
        }
    }
}
