use flate2::read::GzDecoder;
use ratelimit;
use serde::de::DeserializeOwned;
use std::{
	env,
	fmt,
	io::{self, ErrorKind, Read},
	thread,
	time::Duration
};
use ureq::{Agent, AgentBuilder};
use backoff::ExponentialBackoff;
use serde_json;

#[derive(Debug)]
pub enum PolygonError {
	MissingEnv(String),
    RequestError(ureq::Error),
    BackoffError(Box<backoff::Error<PolygonError>>),
    IoError(io::Error),
    SerdeError(serde_json::Error),
    ResponseError(backoff::Error<io::Error>)
}

impl std::error::Error for PolygonError {}

impl fmt::Display for PolygonError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			PolygonError::MissingEnv(e) => write!(f, "invalid config {}", e),
			PolygonError::RequestError(e) => write!(f, "request error {}", e),
			PolygonError::BackoffError(e) => write!(f, "backoff error {}", e),
			PolygonError::IoError(e) => write!(f, "io error {}", e),
			PolygonError::SerdeError(e) => write!(f, "serde error {}", e),
			PolygonError::ResponseError(e) => write!(f, "response error {}", e),
		}
	}
}

pub type Result<T> = std::result::Result<T, PolygonError>;

#[derive(Clone)]
pub struct Client {
	pub agent: Agent,
	pub api_uri: String,
	pub stream_uri: String,
	pub key: String,
	pub ratelimit: u32,
	pub ratelimit_handle: ratelimit::Handle
}

fn make_ratelimit(ratelimit: u32) -> ratelimit::Handle {
	let mut ratelimit = ratelimit::Builder::new()
		.capacity(1)
		.quantum(1)
		.frequency(ratelimit)
		.build();
	let limit = ratelimit.make_handle();
	thread::spawn(move || {
		ratelimit.run();
	});

	limit
}

impl Client {
	pub fn new() -> Result<Client> {
		let ratelimit: u32 = env::var("POLYGON_RATELIMIT")
        // Polygon's API starts ratelimiting at around 100req/s
      .unwrap_or("95".to_string())
      .parse::<u32>()
      .expect("Ratelimit must be an unsigned int");

		let key =
			env::var("POLYGON_KEY").map_err(|_| PolygonError::MissingEnv("POLYGON_KEY".to_string()))?;
		let api_uri = env::var("POLYGON_BASE").unwrap_or(String::from("https://api.polygon.io"));
		let stream_uri = env::var("POLYGON_BASE_WS").unwrap_or(String::from("wss://socket.polygon.io"));
		let agent: Agent = AgentBuilder::new()
			.timeout_read(Duration::from_secs(5))
			.timeout_write(Duration::from_secs(5))
			.build();

		Ok(Self {
			agent,
			key,
			api_uri,
			stream_uri,
			ratelimit,
			ratelimit_handle: make_ratelimit(ratelimit)
		})
	}

	pub fn get_ratelimit(&self) -> u32 { self.ratelimit }

	pub fn get_response<T: DeserializeOwned>(&mut self, uri: &str) -> Result<T> {
        let op = || -> std::result::Result<T, backoff::Error<PolygonError>> {
            self.ratelimit_handle.wait();
            let resp = self
                .agent
                .get(&uri)
                .set("accept-encoding", "gzip")
                .set("authorization", &format!("Bearer {}", self.key))
                .call();
            if let Err(e) = resp {
                return Err(backoff::Error::transient(PolygonError::RequestError(e)));
            }
            let resp = resp.unwrap();

            let status = resp.status();
            if status != 200 {
                return Err(backoff::Error::transient(PolygonError::IoError(io::Error::new(
                    ErrorKind::NotConnected,
                    format!("Server returned {}", status)
                ))));
            }

            let content_encoding = resp.header("content-encoding");
            if content_encoding.is_none() || content_encoding.unwrap() != "gzip" {
                return resp.into_json::<T>().map_err(|e| backoff::Error::permanent(PolygonError::IoError(e)));
            }

            // Decompress
            let mut bytes: Vec<u8> = Vec::new();
            resp
                .into_reader()
                .read_to_end(&mut bytes)
                .map_err(|e| PolygonError::IoError(e))?;

            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut body = String::new();
            decoder.read_to_string(&mut body).unwrap();

            return serde_json::from_str::<T>(&body).map_err(|e| backoff::Error::permanent(PolygonError::SerdeError(e)));
        };
        
        let backoff = ExponentialBackoff::default();
        backoff::retry(backoff, op).map_err(|e| PolygonError::BackoffError(Box::new(e)))
	}
}
