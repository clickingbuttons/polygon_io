use backoff::ExponentialBackoff;
use flate2::read::GzDecoder;
use serde::de::DeserializeOwned;
use serde_json;
use std::{
	env, fmt,
	io::{self, ErrorKind, Read},
	time::Duration
};
use ureq::{Agent, AgentBuilder};

#[derive(Debug)]
pub enum Error {
	MissingEnv(String),
	RequestError(ureq::Error),
	IoError(io::Error),
	SerdeError(serde_json::Error),
	ResponseError(backoff::Error<io::Error>),
	EmptyResponse()
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::MissingEnv(e) => write!(f, "invalid config {}", e),
			Error::RequestError(e) => write!(f, "request error {}", e),
			Error::IoError(e) => write!(f, "io error {}", e),
			Error::SerdeError(e) => write!(f, "serde error {}", e),
			Error::ResponseError(e) => write!(f, "response error {}", e),
			Error::EmptyResponse() => write!(f, "empty response")
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Client {
	pub agent:      Agent,
	pub api_uri:    String,
	pub stream_uri: String,
	pub key:        String
}

impl Client {
	pub fn new() -> Result<Client> {
		let key = env::var("POLYGON_KEY").map_err(|_| Error::MissingEnv("POLYGON_KEY".to_string()))?;
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
			stream_uri
		})
	}

	pub fn get_response<T: DeserializeOwned>(&self, uri: &str) -> Result<T> {
		let op = || -> std::result::Result<T, backoff::Error<Error>> {
			let resp = self
				.agent
				.get(&uri)
				.set("accept-encoding", "gzip")
				.set("authorization", &format!("Bearer {}", self.key))
				.call()
				.map_err(|e| match e {
					// Ureq will raise error here if status >= 400
					ureq::Error::Status(status, _resp) => match status {
						404 => backoff::Error::permanent(Error::EmptyResponse()),
						c => {
							let io_error = Error::IoError(io::Error::new(
								ErrorKind::NotConnected,
								format!("server returned {}", c)
							));
							backoff::Error::permanent(io_error)
						}
					},
					ureq::Error::Transport(e) => {
						backoff::Error::transient(Error::RequestError(ureq::Error::Transport(e)))
					}
				})?;

			if resp.status() != 200 {
				let io_error = Error::IoError(io::Error::new(
					ErrorKind::NotConnected,
					format!("server returned {}", resp.status())
				));
				return Err(backoff::Error::permanent(io_error));
			}

			let content_encoding = resp.header("content-encoding");
			if content_encoding.is_none() || content_encoding.unwrap() != "gzip" {
				return resp
					.into_json::<T>()
					.map_err(|e| Error::IoError(e))
					.map_err(backoff::Error::Permanent);
			}

			// Decompress
			// TODO: capacity based on Content-Length
			let mut bytes: Vec<u8> = Vec::new();
			resp.into_reader().read_to_end(&mut bytes).map_err(|e| {
				eprintln!("3 {}", e);
				return Error::IoError(e);
			})?;

			let mut decoder = GzDecoder::new(&bytes[..]);
			let mut body = String::new();
			decoder.read_to_string(&mut body).unwrap();

			return serde_json::from_str::<T>(&body)
				.map_err(|e| backoff::Error::Permanent(Error::SerdeError(e)));
		};

		let backoff = ExponentialBackoff::default();
		backoff::retry(backoff, op).map_err(|e| match e {
			backoff::Error::Transient {
				err,
				retry_after: _
			} => err,
			backoff::Error::Permanent(err) => err
		})
	}
}
