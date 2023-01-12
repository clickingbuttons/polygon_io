extern crate serde_json;
extern crate ureq;

use crate::{client::{Client, Result, PolygonError}, helpers::*, with_param};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	io::{Error, ErrorKind}
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Ticker {
	#[serde(rename(deserialize = "ticker"))]
	pub symbol: String,
	pub name: String,
	pub market: String,
	pub locale: String,
	pub primary_exchange: Option<String>,
	pub r#type: Option<String>,
	pub active: bool,
	pub currency_name: Option<String>,
	pub cik: Option<String>,
	pub composite_figi: Option<String>,
	pub share_class_figi: Option<String>,
	pub delisted_utc: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickersResponse {
	pub results:    Vec<Ticker>,
	pub count:      usize,
	pub next_url:   Option<String>,
	// For debugging
	pub status:     String,
	pub request_id: String
}

pub struct TickersParams<'a> {
	pub params: HashMap<&'a str, String>
}

impl<'a> TickersParams<'a> {
	with_param!(ticker, &str);

	with_param!(r#type, &str);

	with_param!(market, &str);

	with_param!(exchange, &str);

	with_param!(cusip, &str);

	with_param!(date, &str);

	with_param!(active, bool);

	with_param!(sort, &str);

	with_param!(order, &str);

	with_param!(limit, usize);

	// Undocumented but appears in next_page_path
	with_param!(cursor, &str);

	pub fn new() -> Self {
		Self {
			params: HashMap::with_capacity(8)
		}
	}
}

impl Client {
	pub fn get_tickers(
		&mut self,
		params: Option<&HashMap<&str, String>>
	) -> Result<TickersResponse> {
		let uri = format!(
			"{}/v3/reference/tickers{}",
			self.api_uri,
			make_params(params),
		);

		let resp = self.get_response::<TickersResponse>(&uri)?;

		Ok(resp)
	}

	pub fn get_all_tickers(&mut self, date: &str) -> Result<Vec<Ticker>> {
		let limit: usize = 1000;
		// Use default params since next_page_path does as well
		let mut params = TickersParams::new()
			.market("stocks")
			.limit(limit)
			.order("asc")
			.sort("ticker")
			.date(date);
		let mut res = Vec::<Ticker>::new();
		loop {
			let page = self.get_tickers(Some(&params.params))?;
			res.extend(page.results.into_iter());
			match page.next_url {
				Some(next_url) => {
					let split = next_url.split("cursor=").collect::<Vec<&str>>();
					if split.len() != 2 {
						let msg = format!("no cursor in next_url {}", next_url);
                        let io_error = Error::new(ErrorKind::UnexpectedEof, msg);
						return Err(PolygonError::IoError(io_error));
					}
					let cursor = split[1];
					params = TickersParams::new().cursor(cursor);
				}
				None => break
			};
		}

		Ok(res)
	}
}

#[cfg(test)]
mod tickers {
	use crate::client::Client;

	#[test]
	fn works() {
		let mut client = Client::new().unwrap();
		let resp = client.get_tickers(None).unwrap();
		assert_eq!(resp.results.len(), 100);
	}

	#[test]
	fn works_day() {
		let mut client = Client::new().unwrap();
		let results = client.get_all_tickers("2004-01-02").unwrap();
		assert_eq!(results.len(), 8163);
	}
}
