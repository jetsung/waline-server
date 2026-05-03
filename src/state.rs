use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use crate::{config::Config, db::Db, geoip::GeoIp};

static GLOBAL_GEOIP: OnceLock<Option<Arc<dyn GeoIp>>> = OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Arc<Config>,
    pub rate_limiter: Arc<RateLimiter>,
    #[allow(dead_code)]
    pub geoip: Option<Arc<dyn GeoIp>>,
}

impl AppState {
    pub fn new(db: Db, config: Config, geoip: Option<Arc<dyn GeoIp>>) -> Self {
        // Store geoip globally for sync access
        let _ = GLOBAL_GEOIP.set(geoip.clone());
        
        let ipqps = config.ipqps;
        Self {
            db,
            rate_limiter: Arc::new(RateLimiter::new(ipqps)),
            config: Arc::new(config),
            geoip,
        }
    }
}

/// Get the global GeoIP instance
pub fn get_geoip() -> Option<Arc<dyn GeoIp>> {
    GLOBAL_GEOIP.get().and_then(|g| g.clone())
}

pub struct RateLimiter {
    window_secs: u64,
    counter: Mutex<HashMap<String, (usize, Instant)>>,
}

impl RateLimiter {
    pub fn new(window_secs: u64) -> Self {
        Self { window_secs, counter: Mutex::new(HashMap::new()) }
    }

    /// Returns true if the request is allowed (not rate-limited).
    pub fn check(&self, ip: &str) -> bool {
        let mut map = self.counter.lock().unwrap();
        let window = Duration::from_secs(self.window_secs);
        map.retain(|_, (_, ts)| ts.elapsed() < window);
        match map.get_mut(ip) {
            Some((cnt, _)) => {
                if *cnt >= 1 {
                    false
                } else {
                    *cnt += 1;
                    true
                }
            }
            None => {
                map.insert(ip.to_string(), (1, Instant::now()));
                true
            }
        }
    }
}
