//! # Se621
//! Se621 is a multithreaded e621/e926 downloader written in Rust.
//!
//! ## Usage
//! 1. Run the program once to create the tags file.
//! 2. Add you're tags / pool id's to the tags file.
//! 3. Run the program.
mod e621;

use e621::download;
use e621::file;
use e621::scraper;
use e621::unit;

use std::collections::VecDeque;
extern crate clap;
use clap::{App, Arg};

pub const APP_USER_AGENT: &str = "Se621/0.5.1";
pub const MAX_CHAN_COUNT_TRY: usize = 20;
pub const BANNER: &str = "   _____ ______   ________  ___\n  / ___// ____/  / ___/__ \\<  /\n  \\__ \\/ __/    / __ \\__/ // / \n ___/ / /___   / /_/ / __// /  \n/____/_____/   \\____/____/_/   \n";

#[tokio::main]
async fn main() {
    // Clap command line parser functionality
    let matches = App::new("Se621")
        .version("0.5")
        .author("Asriel <Asriel@dismail.de>")
        .about("Downloads images from e621/e926 concurrently")
        .arg(
            Arg::with_name("tag-file")
                .short("f")
                .long("tag-file")
                .value_name("FILE")
                .help("The file containing the tags you want to download")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sfw")
                .short("s")
                .long("sfw")
                .value_name("SFW")
                .help("Set's the scraper to only download safe for work images")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .value_name("VERBOSE")
                .help("Output more detail during the scraping and downloading process")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .value_name("DIRECTORY")
                .help("The directory to download the files to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tries")
                .short("t")
                .long("tries")
                .value_name("TRIES")
                .help("The amount of times to try downloading a file")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .value_name("WORKERS")
                .help("The amount of threads you want to use to download filse")
                .default_value("8")
                .takes_value(true),
        )
        .get_matches();

    println!("{}", BANNER);

    let mut dir_string = String::new();

    if let Some(dir) = matches.value_of("directory") {
        dir_string = String::from(dir);
    }

    // Create the config struct we will pass to other functions
    let config = unit::Config::new(
        matches.is_present("sfw"),
        matches.is_present("verbose"),
        Some(dir_string),
    );

    // Create the main queue to store Containers in for processing
    let mut queue = VecDeque::<unit::Container>::new();

    let fresh_tags =
        file::read_tags(matches.value_of("tag-file")).expect("[-] Failed to parse tag file");

    file::check_pop(&fresh_tags);

    // Start scraping posts
    println!("[=] Scraping Posts");
    for tag in fresh_tags.general {
        queue.push_back(scraper::build_tag_queue(&tag, &config).await);
    }

    for tag in fresh_tags.pools {
        queue.push_back(
            scraper::build_pool_queue(
                tag.parse::<u64>()
                    .expect("[-] Failed to convert pool id to integer"),
                &config,
            )
            .await,
        );
    }

    for tag in fresh_tags.single_posts {
        queue.push_back(
            scraper::build_single_post(
                tag.parse::<u64>()
                    .expect("[-] Failed to convert pool id to integer"),
                &config,
            )
            .await,
        );
    }

    let num_tries = matches
        .value_of("tries")
        .expect("[-] Failed to parse number of tries")
        .parse::<usize>()
        .expect("[-] Invalid value for number of tries");
    let num_workers = matches
        .value_of("workers")
        .expect("[-] Failed to parse number of workers")
        .parse::<usize>()
        .expect("[-] Invalid value for number of workers");

    // Start downloading files
    println!("\n[=] Downloading Files");
    for q in &mut queue {
        let down = download::Downloader::new(num_tries, num_workers, &mut q.queue);
        down.download(&q.tag_name, &config);
    }
}
