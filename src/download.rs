use crate::unit;
use crossbeam;
use crossbeam::channel;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io;

pub struct Downloader {
    //channel_tx: channel::Sender<unit::Unit>,
    channel_rx: channel::Receiver<unit::Unit>,
    tries: usize,
    jobs: usize,
}

impl Downloader {
    pub fn new(tries: usize, jobs: usize, queue: &mut VecDeque<unit::Unit>) -> Downloader {
        let (tx, rx) = channel::unbounded::<unit::Unit>();

        for u in queue.drain(..) {
            tx.send(u).unwrap();
        }

        Downloader {
            //channel_tx: tx,
            channel_rx: rx,
            tries: tries,
            jobs: jobs,
        }
    }

    pub fn download(&self, tag_dir: &str) {
        crossbeam::thread::scope(|thread_scope| {
            let mut handles = Vec::new();

            for x in 0..self.jobs {
                println!("[!] Thread {} Spawned", x);
                let mut chan_counter = 0;
                let mut retry_counter = 0;
                let cl_chan = self.channel_rx.clone();

                let mut cur_dir = env::current_dir().unwrap();
                cur_dir.push("downloads");
                cur_dir.push(tag_dir);
                fs::create_dir_all(&cur_dir).unwrap();

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

                        while retry_counter < self.tries {
                            let response =
                                down_client.get(cur_unit.url.as_ref().unwrap()).send();
                            if let Err(_) = response {
                                retry_counter += 1;
                                continue;
                            }

                            let resp = match response.unwrap().bytes() {
                                Err(_) => {
                                    retry_counter += 1;
                                    continue;
                                },
                                Ok(resp) => resp,
                            };

                            let mut cur = io::Cursor::new(resp);
                            let mut cur_file = cur_dir.clone();
                            cur_file.push(format!("{}.{}", cur_unit.name, cur_unit.ext));
                            let mut o_file = fs::File::create(&cur_file).unwrap();
                            io::copy(&mut cur, &mut o_file).unwrap();
                            println!("Thread {}: {}.{}", &x, cur_unit.name, cur_unit.ext);
                            break;
                        }
                        retry_counter = 0;
                        chan_counter = 0;
                    }
                });
                handles.push(handle);
            }
            for h in handles {
                h.join().unwrap();
            }
            println!("Finished threads");
        })
        .unwrap();
    }
}
