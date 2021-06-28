use crate::unit;

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::VecDeque;
use url::Url;

/// Struct used for deserializing tag results
#[derive(Deserialize, Debug)]
struct TagPayload {
    posts: Vec<Post>,
}

/// Struct that represents a post
#[derive(Deserialize, Debug)]
struct Post {
    id: u64,
    file: File,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

/// Struct that represents a file
#[derive(Deserialize, Debug)]
struct File {
    ext: String,
    md5: String,
    url: Option<String>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl Iterator for TagPayload {
    type Item = Post;

    fn next(&mut self) -> Option<Self::Item> {
        self.posts.pop()
    }
}

/// Struct that represents a pool
#[derive(Deserialize, Debug)]
struct Pool {
    id: u64,
    name: String,
    post_ids: Vec<u64>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

/// Function to build a queue full of untis for a specfifed tag
pub async fn build_tag_queue(tag: &str, config: &unit::Config) -> unit::Container {
    let mut queue = VecDeque::new();

    // Create the client used for downloading
    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

    let mut head = 0;

    println!("[+] Scraping Tag: {}", tag);

    // e621 uses relative tag_id's we can walk the entire contents of a tag
    // by using the last id on the page as the starting id for the next page
    loop {
        let a_str = format!("a{}", head.to_string());

        let mut url = urlencoding::decode(
            Url::parse_with_params(
                "https://e621.net/posts.json",
                &[("limit", "320"), ("tags", tag), ("page", &a_str)],
            )
            .unwrap()
            .as_str(),
        )
        .unwrap();
        if config.sfw {
            url = urlencoding::decode(
                Url::parse_with_params(
                    "https:///e926.net/posts.json",
                    &[("limit", "320"), ("tags", tag), ("page", &a_str)],
                )
                .unwrap()
                .as_str(),
            )
            .unwrap();
        }

        let mut batch = get_tag_json(&url, &app_client).await;

        if batch.posts.is_empty() {
            break;
        }

        head = batch.posts.get(0).unwrap().id;

        for post in &mut batch {
            queue.push_back(unit::Unit {
                dir_tag: String::from(tag),
                name: post.file.md5,
                ext: post.file.ext,
                url: post.file.url,
            });
        }

        if config.verbose {
            println!("Head: {}", head);
            println!("tag: {}", tag);
            println!("{}", url);
            println!("Size: {}\n", queue.len());
        }
    }

    unit::Container {
        tag_name: String::from(tag),
        queue,
    }
}

/// Function to build a queue for a pool
pub async fn build_pool_queue(pool_id: u64, config: &unit::Config) -> unit::Container {
    println!("[+] Scraping Pool: {}", pool_id);

    let mut queue = VecDeque::new();

    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

    let mut url = urlencoding::decode(
        Url::parse_with_params(
            "https://e621.net/pools.json",
            &[("search[id]", pool_id.to_string())],
        )
        .unwrap()
        .as_str(),
    )
    .unwrap();
    if config.sfw {
        url = urlencoding::decode(
            Url::parse_with_params(
                "https://e926.net/pools.json",
                &[("search[id]", pool_id.to_string())],
            )
            .unwrap()
            .as_str(),
        )
        .unwrap();
    }

    if config.verbose {
        println!("{}\n", url);
    }

    let batch = get_pool_json(&url, &app_client).await;
    let post_ids = batch.get(0).unwrap().post_ids.clone();

    for (counter, id) in post_ids.into_iter().enumerate() {
        build_pool_post(
            id,
            batch.get(0).unwrap().name.clone(),
            counter,
            &mut queue,
            config,
        )
        .await;
    }

    unit::Container {
        tag_name: batch.get(0).unwrap().name.clone(),
        queue,
    }
}

/// Function used to build individual posts from a pool
pub async fn build_pool_post(
    mut post_id: u64,
    dir_tag: String,
    post_num: usize,
    queue: &mut VecDeque<unit::Unit>,
    config: &unit::Config,
) {
    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

    let old_id = post_id;
    post_id -= 1;

    let mut url = urlencoding::decode(
        Url::parse_with_params(
            "https://e621.net/posts.json",
            &[
                ("limit", "1"),
                ("page", &format!("{}{}", "a", post_id.to_string())),
            ],
        )
        .unwrap()
        .as_str(),
    )
    .unwrap();
    if config.sfw {
        url = urlencoding::decode(
            Url::parse_with_params(
                "https://e926.net/posts.json",
                &[
                    ("limit", "1"),
                    ("page", &format!("{}{}", "a", post_id.to_string())),
                ],
            )
            .unwrap()
            .as_str(),
        )
        .unwrap();
    }
    let batch = get_tag_json(&url, &app_client).await;

    let post = batch.posts.get(0).unwrap();

    if post.id != old_id {
        return;
    }

    queue.push_back(unit::Unit {
        dir_tag,
        name: post_num.to_string(),
        ext: post.file.ext.clone(),
        url: post.file.url.clone(),
    });
}

/// Function to handle downloading individual posts
pub async fn build_single_post(mut post_id: u64, config: &unit::Config) -> unit::Container {
    println!("[+] Scraping Single Post: {}", post_id);
    let mut queue = VecDeque::new();

    post_id -= 1;

    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

    let mut url = urlencoding::decode(
        Url::parse_with_params(
            "https://e621.net/posts.json",
            &[
                ("limit", "1"),
                ("page", &format!("{}{}", "a", post_id.to_string())),
            ],
        )
        .unwrap()
        .as_str(),
    )
    .unwrap();
    if config.sfw {
        url = urlencoding::decode(
            Url::parse_with_params(
                "https://e926.net/posts.json",
                &[
                    ("limit", "1"),
                    ("page", &format!("{}{}", "a", post_id.to_string())),
                ],
            )
            .unwrap()
            .as_str(),
        )
        .unwrap();
    }
    let batch = get_tag_json(&url, &app_client).await;

    let post = batch.posts.get(0).unwrap();

    queue.push_back(unit::Unit {
        dir_tag: String::from(""),
        name: post.file.md5.clone(),
        ext: post.file.ext.clone(),
        url: post.file.url.clone(),
    });

    // TODO: This is dumb, we only need to allocate a single post here so we shouldn't be using a VecDeque fix later
    unit::Container {
        tag_name: post_id.to_string(),
        queue,
    }
}

async fn get_tag_json(url: &str, client: &reqwest::Client) -> TagPayload {
    client
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<TagPayload>()
        .await
        .unwrap()
}

async fn get_pool_json(url: &str, client: &reqwest::Client) -> Vec<Pool> {
    client
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<Vec<Pool>>()
        .await
        .unwrap()
}
