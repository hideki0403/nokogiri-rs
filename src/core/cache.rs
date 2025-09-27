use std::process;
use once_cell::sync::Lazy;
use redis::{Client, Commands};
use xxhash_rust::xxh64::xxh64;
use crate::config::CONFIG;

static REDIS_CLIENT: Lazy<Option<Client>> = Lazy::new(|| {
    let cache_config = &CONFIG.cache;
    if !cache_config.enabled {
        tracing::info!("Cache is disabled");
        return None;
    }

    let mut connection_url = "redis://".to_string();
    if cache_config.username.is_some() || cache_config.password.is_some() {
        let username = cache_config.username.as_ref().map_or("", String::as_str);
        let password = cache_config.password.as_ref().map_or("", String::as_str);
        connection_url.push_str(&format!("{}:{}@", username, password));
    }

    connection_url.push_str(&format!("{}:{}", cache_config.host, cache_config.port));
    if let Some(db) = cache_config.db {
        connection_url.push_str(&format!("/{}", db));
    }

    let client = match Client::open(connection_url) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create Redis client: {}", e);
            process::exit(1);
        }
    };

    // Test connection
    match client.get_connection() {
        Ok(mut conn) => {
            if let Err(e) = conn.ping::<String>() {
                tracing::error!("Failed to connect to Redis server: {}", e);
                process::exit(1);
            }
            tracing::info!("Connected to Redis server");
            Some(client)
        }
        Err(e) => {
            tracing::error!("Failed to get Redis connection: {}", e);
            process::exit(1);
        }
    }
});

fn gen_key(category: &str, identifier: &str) -> String {
    let mut key = String::new();

    if let Some(prefix) = &CONFIG.cache.prefix {
        key.push_str(prefix);
        key.push(':');
    }

    key.push_str(&format!("nokogiri:{}:{}", category, identifier));
    key
}

pub fn get_summarize_cache(url: &str) -> Option<String> {
    let mut connection = REDIS_CLIENT.as_ref()?.get_connection().ok()?;
    let key = gen_key("summarize", xxh64(url.as_bytes(), 0).to_string().as_str());
    tracing::debug!("Checking cache for key: {}", key);
    connection.get(&key).ok()
}

pub fn set_summarize_cache(url: &str, content: &str, ttl: &u64) {
    if ttl == &0 {
        tracing::debug!("TTL is 0, not setting cache");
        return;
    }

    let mut connection = match REDIS_CLIENT.as_ref().and_then(|c| c.get_connection().ok()) {
        Some(conn) => conn,
        None => return,
    };

    let key = gen_key("summarize", xxh64(url.as_bytes(), 0).to_string().as_str());
    let mut ttl = ttl;

    if ttl > &86400 {
        tracing::debug!("TTL is greater than 86400 seconds (1 day), capping to 86400");
        ttl = &86400;
    }

    tracing::debug!("Setting cache for key: {} with TTL: {} seconds", key, ttl);
    match connection.set_ex::<&String, &str, String>(&key, content, *ttl) {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to set cache for key {}: {}", key, e);
            return;
        }
    };
}
