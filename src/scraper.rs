use crate::unit;

use url::Url;
use std::time;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::VecDeque;


#[derive(Deserialize, Debug)]
struct TagPayload {
    posts: Vec<Post>,
}

#[derive(Deserialize, Debug)]
struct Post {
    id: u64,
    file: File,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

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

#[derive(Deserialize, Debug)]
struct Pool {
    id: u64,
    name: String,
    post_ids: Vec<u64>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}




pub async fn build_tag_queue(tag: &String, queue: &mut VecDeque<unit::Unit>) {
    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

    let mut head = 0;

    loop {
        println!("Head: {}", head);
        let a_str = format!("a{}", head.to_string());
        println!("tag: {}", tag);
        let url = urlencoding::decode(Url::parse_with_params("https://e621.net/posts.json", &[("limit", "320"), ("tags", &tag), ("page", &a_str)]).unwrap().as_str()).unwrap();
        println!("{}", url);
        let mut batch = get_tag_json(&url, &app_client).await;

        if batch.posts.len() == 0 {
            return;
        }


        head = batch.posts.get(0).unwrap().id.clone();

        for post in &mut batch {
            queue.push_back(unit::Unit {
                dir_tag: tag.clone(),
                name: post.file.md5,
                ext: post.file.ext,
                url: post.file.url,
            });
        }
        tokio::time::sleep(time::Duration::from_millis(0)).await;
        println!("Size: {}", queue.len());
    }
}

pub async fn build_pool_queue(pool_id: u64, queue: &mut VecDeque<unit::Unit>) {
    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

    let url = urlencoding::decode(Url::parse_with_params("https://e621.net/pools.json", &[("search[id]", pool_id.to_string())]).unwrap().as_str()).unwrap();
    println!("{}", url);
    let batch = get_pool_json(&url, &app_client).await;
    let post_ids = batch.get(0).unwrap().post_ids.clone();
    
    
    let mut counter = 0;
    for id in post_ids {
        build_single_post(id, batch.get(0).unwrap().name.clone(), counter, queue).await;
        counter += 1;
    }
    

}

pub async fn build_single_post(mut post_id: u64, dir_tag: String, post_num: u64, queue: &mut VecDeque<unit::Unit>) {
    let app_client = reqwest::Client::builder()
        .user_agent(crate::APP_USER_AGENT)
        .build()
        .unwrap();

        let old_id = post_id;
        post_id -= 1;

        let url = urlencoding::decode(Url::parse_with_params("https://e621.net/posts.json", &[("limit", "1"), ("page", &format!("{}{}", "a", post_id.to_string()))]).unwrap().as_str()).unwrap();
        let batch = get_tag_json(&url, &app_client).await;

        let post = batch.posts.get(0).unwrap().clone();

        if post.id != old_id {
            return;
        }

        queue.push_back(unit::Unit {
            dir_tag: dir_tag,
            name: post_num.to_string(),
            ext: post.file.ext.clone(),
            url: post.file.url.clone(),
        });
        
}


async fn get_tag_json(url: &str, client: &reqwest::Client) -> TagPayload {
    client.get(url)
        .send()
        .await
        .unwrap()
        .json::<TagPayload>()
        .await
        .unwrap()
}

async fn get_pool_json(url: &str, client: &reqwest::Client) -> Vec<Pool> {
    client.get(url)
        .send()
        .await
        .unwrap()
        .json::<Vec<Pool>>()
        .await
        .unwrap()
}