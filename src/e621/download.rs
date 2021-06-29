use crate::unit;
use crossbeam;
use crossbeam::channel;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io;

/// A struct used for storing information needed to download files
pub struct Downloader {
    //channel_tx: channel::Sender<unit::Unit>,
    channel_rx: channel::Receiver<unit::Unit>,
    tries: usize,
    jobs: usize,
    len: usize,
}

impl Downloader {
    pub fn new(tries: usize, jobs: usize, queue: &mut VecDeque<unit::Unit>) -> Downloader {
        let queue_len = queue.len();
        let (tx, rx) = channel::unbounded::<unit::Unit>();

        for u in queue.drain(..) {
            tx.send(u).unwrap();
        }

        Downloader {
            //channel_tx: tx,
            channel_rx: rx,
            tries,
            jobs,
            len: queue_len,
        }
    }

    /// The function which downloads all of the images to a specified directory
    pub fn download(&self, tag_dir: &str, config: &unit::Config) {
        crossbeam::thread::scope(|thread_scope| {
            let mut handles = Vec::new();

            // Setup the directory to download into
            let mut cur_dir = std::path::PathBuf::new();

            if let Some(down_dir) = &config.directory {
                cur_dir.push(down_dir);
            } else {
                cur_dir = env::current_dir().unwrap();
            }

            if config.sfw {
                cur_dir.push("sfw-downloads")
            } else {
                cur_dir.push("downloads");
            }

            if self.len > 1 {
                cur_dir.push(&tag_dir);
            }

            println!("[+] Downloading / Updating: {}", tag_dir);

            // Setup number of specifed threads for downloaading
            for x in 0..self.jobs {
                let cur_dir = cur_dir.clone();

                if config.verbose {
                    println!("[!] Thread {} Spawned", x);
                }

                // Setup variables for counting the number of attempts made
                let mut chan_counter = 0;
                let mut retry_counter = 0;
                let cl_chan = self.channel_rx.clone();

                fs::create_dir_all(&cur_dir).expect("[-] Failed to create tag directory");

                // Workers start executing here
                let handle = thread_scope.spawn(move |_| {
                    let down_client = reqwest::blocking::ClientBuilder::new()
                        .user_agent(crate::APP_USER_AGENT)
                        .build()
                        .unwrap();

                    while chan_counter < crate::MAX_CHAN_COUNT_TRY {
                        let cur_unit = match cl_chan.try_recv() {
                            Err(_) => {
                                chan_counter += 1;
                                continue;
                            }
                            Ok(cur_unit) => cur_unit,
                        };

                        if cur_unit.url.is_none() {
                            continue;
                        }

                        let mut cur_file = cur_dir.clone();
                        cur_file.push(format!("{}.{}", cur_unit.name, cur_unit.ext));
                        if cur_file.exists() {
                            continue;
                        }

                        // Try and download a file a certain amount of times
                        while retry_counter < self.tries {
                            let response = down_client.get(cur_unit.url.as_ref().unwrap()).send();
                            if response.is_err() {
                                retry_counter += 1;
                                continue;
                            }

                            let resp = match response.unwrap().bytes() {
                                Err(_) => {
                                    retry_counter += 1;
                                    continue;
                                }
                                Ok(resp) => resp,
                            };

                            // Write the file to disk
                            let mut cur = io::Cursor::new(resp);
                            let mut o_file = fs::File::create(&cur_file).unwrap();
                            io::copy(&mut cur, &mut o_file).unwrap();

                            if config.verbose {
                                println!("Thread {}: {}.{}", &x, cur_unit.name, cur_unit.ext);
                            }

                            break;
                        }
                        retry_counter = 0;
                        chan_counter = 0;
                    }
                });
                handles.push(handle);
            }

            // Wait until all threads are complete
            for h in handles {
                h.join().unwrap();
            }
            if config.verbose {
                println!("Finished threads");
            }
        })
        .unwrap();
    }
}
