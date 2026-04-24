use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use serde_json::Value;
use waline_core::config::Config;
use waline_db::adapter::DatabaseAdapter;

#[derive(Debug)]
pub struct RateLimiter {
    qps: u64,
    counter: Mutex<HashMap<String, (usize, Instant)>>,
}

impl RateLimiter {
    pub fn new(qps: u64) -> Self {
        RateLimiter {
            qps,
            counter: Mutex::new(HashMap::new()),
        }
    }
    pub fn check_and_update(&self, client_ip: &str, count: usize) -> bool {
        let mut counter = self.counter.lock().unwrap();
        counter.retain(|_, &mut (_, timestamp)| timestamp.elapsed() < Duration::from_secs(self.qps));
        match counter.get_mut(client_ip) {
            Some((cnt, timestamp)) => {
                if *cnt >= count {
                    false
                } else {
                    *cnt += 1;
                    *timestamp = Instant::now();
                    true
                }
            }
            None => {
                counter.insert(client_ip.to_string(), (1, Instant::now()));
                true
            }
        }
    }
}

#[derive(Clone)]
pub struct CommentCache {
    pub cache: Arc<Mutex<HashMap<(String, i32), Value>>>,
}

impl CommentCache {
    pub fn new() -> Self {
        CommentCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: String, page: i32) -> Option<Value> {
        self.cache.lock().unwrap().get(&(path, page)).cloned()
    }

    pub fn insert(&mut self, path: String, page: i32, data: Value) {
        self.cache.lock().unwrap().insert((path, page), data);
    }

    pub fn invalidate(&self, path: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.retain(|(old_path, _), _| old_path != path);
    }
}

/// Shared application state
pub struct AppState {
    pub config: Config,
    pub db: Arc<dyn DatabaseAdapter>,
    pub rate_limiter: Arc<RateLimiter>,
    pub comment_cache: Arc<Mutex<CommentCache>>,
}

impl AppState {
    pub fn new(config: Config, db: Arc<dyn DatabaseAdapter>) -> Self {
        Self {
            config: config.clone(),
            db,
            rate_limiter: Arc::new(RateLimiter::new(config.ipqps.unwrap_or(60))),
            comment_cache: Arc::new(Mutex::new(CommentCache::new())),
        }
    }
}
