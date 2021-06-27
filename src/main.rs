mod scraper;
mod download;
mod unit;
use std::collections::VecDeque;
extern crate clap;
use clap::{Arg, App, SubCommand};


pub const APP_USER_AGENT: &str = "Se621/0.4.0";
pub const MAX_CHAN_COUNT_TRY: usize = 20;

#[tokio::main]
async fn main() {
    let matches = App::new("Se621")
        .version("0.4")
        .about("Downloads images from e621 concurrently")
        .get_matches();

    let mut queue = VecDeque::new();
    scraper::build_tag_queue(&String::from("fox"), &mut queue).await;
    println!("Queue: {}", queue.len());


    let down = download::Downloader::new(10, 8, &mut queue);
    down.download("fox");



}