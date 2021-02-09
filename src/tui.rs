use std::collections::HashMap;

use crate::module_handler::*;
use crate::shelf::*;

pub struct TUI<'lt> {
    config: &'lt HashMap<String, String>,
    shelf: &'lt Shelf,
    module_handler: &'lt ModuleHandler,
}

impl<'lt> TUI<'lt> {
    pub fn new<'a>(
        config: &'lt HashMap<String, String>,
        shelf: &'lt mut Shelf,
        module_handler: &'lt ModuleHandler,
    ) -> TUI<'lt> {
        TUI {
            config,
            shelf,
            module_handler,
        }
    }

    pub fn start(&mut self) {
        // TODO
        println!("TUI isn't implemented yet!");
    }
}
