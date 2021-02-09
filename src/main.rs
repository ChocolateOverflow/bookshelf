pub mod fsio;
pub mod module_handler;
pub mod shelf;
pub mod tui;

use clap::{load_yaml, App};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

use fsio::*;
use module_handler::*;
use shelf::*;
use tui::*;

fn get_item_dir(data_root: &String, module: &str, code: &str) -> String {
    let mut pb = PathBuf::from(&data_root);
    pb.push(module);
    pb.push(code);
    return pb.as_path().to_str().unwrap().to_string();
}

fn add_by_url(
    shelf: &mut Shelf,
    module_handler: &ModuleHandler,
    url: &str,
    data_root: Option<&String>,
    verbose: bool,
) {
    if let Some(module) = module_handler.derive_module(url) {
        if let Some(code) = module_handler.derive_code(module, url) {
            add_by_code(
                shelf,
                &module_handler,
                module,
                code.as_str(),
                data_root,
                verbose,
            );
        }
    }
}

fn add_by_code(
    shelf: &mut Shelf,
    module_handler: &ModuleHandler,
    module: &str,
    code: &str,
    data_root: Option<&String>,
    verbose: bool,
) {
    if shelf.has_item(&module, &code) {
        if verbose {
            println!("Item {}/{} already indexed", &module, &code);
        }
    } else {
        // Get metadata
        if let Some(metadata) = module_handler.get_metadata(&module, &code) {
            // Print item (verbose)
            if verbose {
                println!(
                    "Adding item: {}/{}\n\tTitle: {}\n\tAuthors: {}\n\tTags: {}",
                    &module, &code, &metadata.0, &metadata.1, metadata.2
                );
            }
            // title
            let title = metadata.0;
            // authors
            let mut authors: HashSet<String> = HashSet::new();
            for author in metadata.1.split(",") {
                authors.insert(author.to_string());
            }
            // tags
            let mut tags: HashSet<String> = HashSet::new();
            for tag in metadata.2.split(",") {
                tags.insert(tag.to_string());
            }
            // Construct item
            shelf.add_item(&module, &code, title, authors, tags);
            // Download if data_root is set
            if let Some(data_root) = data_root {
                let dest_dir: String = get_item_dir(data_root, &module, &code);
                module_handler.download(&module, &code, &dest_dir);
            }
        }
    }
}

/// Add item to index and optionally download it.
/// Set data_root to None to skip download, or set it to Some() to download
fn add_item(
    shelf: &mut Shelf,
    module_handler: &ModuleHandler,
    url: Option<&str>,
    url_file: Option<&str>,
    module: Option<&str>,
    code: Option<&str>,
    code_file: Option<&str>,
    data_root: Option<&String>,
    verbose: bool,
) {
    if let Some(url) = url {
        // shelf add|download -u
        add_by_url(shelf, &module_handler, url, data_root, verbose);
    } else if let Some(url_file) = url_file {
        // shelf add|download -U
        match File::open(url_file) {
            Ok(file) => {
                for line in BufReader::new(file).lines() {
                    match line {
                        Ok(url) => {
                            add_by_url(shelf, &module_handler, url.as_str(), data_root, verbose)
                        }
                        Err(e) => {
                            println!("Error reading URL file: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error reading URL file: {}", e);
            }
        }
    } else if let Some(module) = module {
        if let Some(code) = code {
            // shelf add|download -m MODULE -c CODE
            add_by_code(shelf, &module_handler, module, code, data_root, verbose);
        } else if let Some(code_file) = code_file {
            // shelf add|download -m MODULE -C CODE_FILE
            match File::open(code_file) {
                Ok(file) => {
                    for line in BufReader::new(file).lines() {
                        match line {
                            Ok(code) => add_by_code(
                                shelf,
                                &module_handler,
                                module,
                                code.as_str(),
                                data_root,
                                verbose,
                            ),
                            Err(e) => println!("Error reading code file: {}", e),
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading code file: {}", e);
                }
            }
        } else {
            println!("Error: 'add|download -m MODULE' also requires '-c CODE' or '-C CODE_FILE'")
        }
    } else {
        println!(
            "Error: 'add|download' requires either '-u|-U' or '-m MODULE -c CODE|-C CODE_FILE'"
        );
    }
}

fn cli_print_item(shelf: &Shelf, module: &str, code: &str) {
    if let Some(item) = shelf.get_item(module, code) {
        let (i_title, i_authors, i_tags) = item.export();
        println!("Title: {}", i_title);
        let mut authors = String::new();
        for author in i_authors {
            authors.push_str(author);
            authors.push_str(", ");
        }
        authors.pop();
        authors.pop();
        println!("Authors: {}", authors);
        let mut tags = String::new();
        for tag in i_tags {
            tags.push_str(tag);
            tags.push_str(", ");
        }
        tags.pop();
        tags.pop();
        println!("Authors: {}", tags);
    } else {
    }
}

fn main() {
    /***** Parse arguments and load config *****/
    let arg_file = load_yaml!("args.yaml");
    let args = App::from(arg_file).get_matches();
    let mut config: HashMap<String, String> = HashMap::new();
    {
        let config = &mut config;
        if let Some(c) = args.value_of("config") {
            let path_to_config = PathBuf::from(c);
            config.extend(load_config(&path_to_config));
        } else {
            let mut home_dir = dirs::home_dir();
            match &mut home_dir {
                Some(h) => {
                    h.push(".config/shelf/shelf.yaml");
                    config.extend(load_config(&h))
                }
                _ => println!("Error getting home dir"),
            }
        }
    }
    // Create necessary directories
    create_dirs(
        &PathBuf::from(config.get("data_dir").unwrap()),
        &PathBuf::from(config.get("modules_dir").unwrap()),
    );
    let verbose: bool = { args.is_present("verbose") };

    /***** Initialize shelf and handlers *****/
    // These can be unwrap'd safely because load_config guarantees the entries
    let path_to_index = PathBuf::from(config.get("index_file").unwrap());
    let path_to_modules = PathBuf::from(config.get("modules_dir").unwrap());
    let mut shelf: Shelf = load_shelf(&path_to_index);
    let module_handler = ModuleHandler::new(&path_to_modules);

    /***** main *****/
    match args.subcommand() {
        Some(("modules", _args)) => {
            for i in module_handler.list_modules().iter() {
                println!("{}", i);
            }
        }
        Some(("add", args)) => {
            add_item(
                &mut shelf,
                &module_handler,
                args.value_of("url"),
                args.value_of("url_file"),
                args.value_of("module"),
                args.value_of("code"),
                args.value_of("code_file"),
                None,
                verbose,
            );
        }

        Some(("download", args)) => {
            add_item(
                &mut shelf,
                &module_handler,
                args.value_of("url"),
                args.value_of("url_file"),
                args.value_of("module"),
                args.value_of("code"),
                args.value_of("code_file"),
                config.get("data_dir"),
                verbose,
            );
        }

        Some(("search", args)) => {
            for (m, c) in shelf.search_item(
                None,
                args.value_of("title"),
                args.value_of("authors"),
                args.value_of("tags"),
                args.value_of("blacklist"),
                args.is_present("broad_search"),
                args.is_present("favorite"),
            ) {
                println!("{}/{}", &m, &c);
                if verbose {
                    cli_print_item(&shelf, &m, &c);
                }
            }
        }

        Some(("rm", args)) => {
            for (m, c) in shelf
                .search_item(
                    args.value_of("module"),
                    args.value_of("title"),
                    args.value_of("authors"),
                    args.value_of("tags"),
                    args.value_of("blacklist"),
                    args.is_present("broad_search"),
                    args.is_present("favorite"),
                )
                .iter()
            {
                shelf.remove_item(m, c);
            }
        }

        Some(("pull", args)) => {
            for (m, c) in shelf.search_item(
                args.value_of("module"),
                args.value_of("title"),
                args.value_of("authors"),
                args.value_of("tags"),
                args.value_of("blacklist"),
                args.is_present("broad_search"),
                args.is_present("favorite"),
            ) {
                module_handler.download(
                    &m[..],
                    &c[..],
                    &String::from(config.get("data_dir").unwrap()),
                );
            }
        }

        Some(("info", args)) => {
            cli_print_item(
                &shelf,
                args.value_of("module").unwrap(),
                args.value_of("code").unwrap(),
            );
        }

        Some(("edit", args)) => {
            shelf.edit_item(
                args.value_of("module"),
                args.value_of("code"),
                args.value_of("title"),
                args.value_of("authors"),
                args.value_of("tags"),
                args.is_present("favorite"),
            );
        }

        None => {
            // Start TUI if no argument is given
            let mut tui = TUI::new(&config, &mut shelf, &module_handler);
            tui.start();
        }

        _ => {
            println!("Invalid subcommand");
        }
    }

    /***** Save and exit *****/
    save_shelf(&shelf, &path_to_index);
}
