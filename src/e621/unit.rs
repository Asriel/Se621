use std::collections::VecDeque;

/// Struct for holding information about a sinble post that needs downloading
#[derive(Debug)]
pub struct Unit {
    pub dir_tag: String,
    pub name: String,
    pub ext: String,
    pub url: Option<String>,
}

/// Struct for holding the tag name and the queue holding all the posts for that tag
#[derive(Debug)]
pub struct Container {
    pub tag_name: String,
    pub queue: VecDeque<Unit>,
}

/// Struct for holding config information so it can be easly passed around
#[derive(Debug)]
pub struct Config {
    pub sfw: bool,
    pub verbose: bool,
    pub directory: Option<String>,
}

impl Config {
    pub fn new(sfw: bool, verbose: bool, directory: Option<String>) -> Self {
        Config {
            sfw,
            verbose,
            directory,
        }
    }
}
