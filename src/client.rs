use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Serialize, Deserialize, Clone)]
pub struct Client {
  #[serde(default)]
  pub api_uri:    String,
  #[serde(default)]
  pub stream_uri: String,
  pub key:        String
}

impl Client {
  fn merge(&mut self, other: Client) {
    if !other.api_uri.is_empty() {
      self.api_uri = other.api_uri;
    }
    if !other.stream_uri.is_empty() {
      self.stream_uri = other.stream_uri;
    }
    if !other.key.is_empty() {
      self.key = other.key;
    }
  }

  pub fn open(path: &str) -> Client {
    let mut res: Client = Default::default();
    if let Ok(config_str) = fs::read_to_string(path) {
      match serde_json::from_str::<Client>(&config_str) {
        Ok(config) => res.merge(config),
        Err(e) => {
          eprintln!(
            "Error loading {}, falling back to default config: {}",
            path, e
          );
        }
      };
    }
    res.merge(Client {
      key:        env::var("POLYGON_KEY").unwrap_or_default(),
      api_uri:    env::var("POLYGON_API_URI").unwrap_or_default(),
      stream_uri: env::var("POLYGON_STREAM_URI").unwrap_or_default()
    });

    res
  }

  pub fn new() -> Client { Client::open("polygon.json") }
}

impl Default for Client {
  fn default() -> Self {
    Self {
      api_uri:    String::from("https://api.polygon.io"),
      stream_uri: String::from("wss://socket.polygon.io"),
      key:        String::new()
    }
  }
}
