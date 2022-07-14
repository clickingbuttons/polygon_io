use std::io::Read;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::{env, fs,
  io::{Error, ErrorKind},
  thread
};
use ratelimit;

#[derive(Serialize, Deserialize, Clone)]
pub struct Client {
  #[serde(default)]
  pub api_uri:           String,
  #[serde(default)]
  pub stream_uri:        String,
  pub key:               String,
  #[serde(default = "default_ratelimit")]
  pub ratelimit:         u32,
  #[serde(skip)]
  pub ratelimit_handle:  Option<ratelimit::Handle>,
}

// Polygon's API starts ratelimiting at 100req/s
const DEFAULT_RATELIMIT: u32 = 95;

fn default_ratelimit() -> u32 { DEFAULT_RATELIMIT }

fn make_ratelimit(ratelimit: u32) -> ratelimit::Handle {
  let mut ratelimit = ratelimit::Builder::new()
    .capacity(1)
    .quantum(1)
    .frequency(ratelimit)
    .build();
  let limit = ratelimit.make_handle();
  thread::spawn(move || { ratelimit.run(); });

  limit
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
    if other.ratelimit != 0 {
      self.ratelimit = other.ratelimit;
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

    let ratelimit: u32 = env::var("POLYGON_RATELIMIT")
      .unwrap_or(String::from("0"))
      .parse::<u32>()
      .expect("Ratelimit must be an unsigned int");

    res.merge(Client {
      key:               env::var("POLYGON_KEY").unwrap_or_default(),
      api_uri:           env::var("POLYGON_API_URI").unwrap_or_default(),
      stream_uri:        env::var("POLYGON_STREAM_URI").unwrap_or_default(),
      ratelimit,
      ratelimit_handle:  None
    });
    res.ratelimit_handle = Some(make_ratelimit(res.ratelimit));

    res
  }

  pub fn new() -> Client { Client::open("polygon.json") }

  pub fn get_response<T: DeserializeOwned>(&mut self, uri: &str) -> std::io::Result<T> {
    self.ratelimit_handle.as_mut().unwrap().wait();
    let resp = ureq::get(&uri)
			.set("accept-encoding", "gzip")
			.set("authorization", &format!("Bearer {}", self.key))
			.call();
    if resp.is_err() {
      return Err(Error::new(
        ErrorKind::TimedOut,
        format!("{:?}", resp.unwrap_err())
      ));
    }
    let resp = resp.unwrap();

    let status = resp.status();
    if status != 200 {
      return Err(Error::new(
        ErrorKind::NotConnected,
        format!("Server returned {}", status)
      ));
    }

    let content_encoding = resp.header("content-encoding");
    if content_encoding.is_none() || content_encoding.unwrap() != "gzip" {
      return Ok(resp.into_json::<T>()?);
    }

    // Decompress
    let expected_len = 2_500_000;
    let mut bytes: Vec<u8> = Vec::with_capacity(expected_len);
    resp.into_reader()
        .take(10_000_000)
        .read_to_end(&mut bytes)?;

    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut body = String::new();
    decoder.read_to_string(&mut body).unwrap();

    Ok(serde_json::from_str::<T>(&body)?)
  }
}

impl Default for Client {
  fn default() -> Self {
    Self {
      api_uri:           String::from("https://api.polygon.io"),
      stream_uri:        String::from("wss://socket.polygon.io"),
      key:               String::new(),
      ratelimit:         DEFAULT_RATELIMIT,
      ratelimit_handle:  None
    }
  }
}
