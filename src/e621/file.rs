use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path;

/// Structure for representing the values recovered from the tags file
#[derive(Debug)]
pub struct TagStore {
    pub general: Vec<String>,
    pub pools: Vec<String>,
    pub single_posts: Vec<String>,
}

impl TagStore {
    fn new() -> Self {
        TagStore {
            general: Vec::new(),
            pools: Vec::new(),
            single_posts: Vec::new(),
        }
    }
}

/// Function that handles parsing the tags file
pub fn read_tags(tag_file: Option<&str>) -> io::Result<TagStore> {
    let tag_filepath = check_file_path(tag_file);
    let file = fs::File::open(tag_filepath)?;
    let reader = io::BufReader::new(file);

    let mut stor = TagStore::new();
    let mut last = &mut stor.general;

    for line in reader.lines() {
        let line = line.unwrap();

        if line.starts_with('#') {
            continue;
        }

        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') {
            match line.as_str() {
                "[general]" => {
                    last = &mut stor.general;
                }
                "[pools]" => {
                    last = &mut stor.pools;
                }
                "[single-post]" => last = &mut stor.single_posts,
                e => panic!("[-] Problem with tag file: {}", e),
            }
            continue;
        }

        let line = line.trim();
        last.push(String::from(line));
    }

    Ok(stor)
}

/// Function that handles finding or creating the tags file
pub fn check_file_path(tag_file: Option<&str>) -> path::PathBuf {
    let mut tag_filepath = path::PathBuf::new();

    if tag_file.is_none() {
        tag_filepath = env::current_dir().expect("Couldn't get current directory path");
        tag_filepath.push("tags");

        if !tag_filepath.exists() {
            let data = "# This file contains the tags and pools the program will download\n# Lines begginning with # are comments\n# Insert tags you wish to download in the appropriate group\n\n[general]\n\n[pools]\n\n[single-post]\n\n";

            fs::File::create("tags").expect("[-] Could not create tag file");
            fs::write(tag_filepath, data).expect("[-] Failed to write default config to tags file");
            println!("[+] Created tag file please populate it with values");
            std::process::exit(0);
        }
    }

    if let Some(tag_file) = tag_file {
        tag_filepath.push(tag_file);
    }

    if !tag_filepath.exists() {
        println!("[-] Specified tag file doesn't exist");
        std::process::exit(1);
    }

    tag_filepath
}

/// Function to check wheather any tags where retrived from the tags file if not tell the user to add some
pub fn check_pop(tags: &TagStore) {
    if tags.general.is_empty() && tags.pools.is_empty() && tags.single_posts.is_empty() {
        println!("[-] Please add at least one tag to the tag file");
        std::process::exit(1);
    }
}
