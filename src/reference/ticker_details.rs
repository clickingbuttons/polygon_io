extern crate serde_json;
extern crate ureq;

use crate::{
	client::{Client, Result},
	helpers::*,
	with_param
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Address {
	pub address1:    Option<String>,
	pub address2:    Option<String>,
	pub city:        Option<String>,
	pub state:       Option<String>,
	pub country:     Option<String>,
	pub postal_code: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Branding {
	pub icon_url: Option<String>,
	pub logo_url: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerDetail {
	pub ticker: String,
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
	pub market_cap: Option<f64>,
	pub phone_number: Option<String>,
	//#[serde(flatten)]
	// pub address: Option<Address>,
	pub description: Option<String>,
	pub sic_code: Option<String>,
	pub sic_description: Option<String>,
	pub ticker_root: Option<String>,
	pub homepage_url: Option<String>,
	pub total_employees: Option<u32>,
	pub list_date: Option<String>,
	//#[serde(flatten)]
	// pub branding: Option<Branding>,
	pub share_class_shares_outstanding: Option<f64>,
	pub weighted_shares_outstanding: Option<f64>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickersResponse {
	pub results:    TickerDetail,
	// For debugging
	pub status:     String,
	pub request_id: String
}

pub struct TickerDetailsParams<'a> {
	pub params: HashMap<&'a str, String>
}

impl<'a> TickerDetailsParams<'a> {
	with_param!(date, &str);

	pub fn new() -> Self {
		Self {
			params: HashMap::with_capacity(2)
		}
	}
}

impl Client {
	pub fn get_ticker_details(
		&self,
		ticker: &str,
		params: Option<&HashMap<&str, String>>
	) -> Result<TickersResponse> {
		let uri = format!(
			"{}/v3/reference/tickers/{}{}",
			self.api_uri,
			ticker,
			make_params(params),
		);

		let resp = self.get_response::<TickersResponse>(&uri)?;

		Ok(resp)
	}
}

#[cfg(test)]
mod tickers {
	use crate::{
		client::{Client, PolygonError},
		reference::ticker_details::TickerDetailsParams
	};

	#[test]
	fn works() {
		let client = Client::new().unwrap();
		let resp = client.get_ticker_details("AAPL", None).unwrap();
		assert_eq!(resp.results.market, "stocks");
	}

	#[test]
	fn works_day() {
		let client = Client::new().unwrap();
		let params = TickerDetailsParams::new().date("2004-01-02").params;
		let resp = client.get_ticker_details("AAPL", Some(&params)).unwrap();
		assert_eq!(resp.results.market, "stocks");
	}

	#[test]
	fn works_empty() {
		let client = Client::new().unwrap();
		let params = TickerDetailsParams::new().date("2004-01-02").params;
		let resp = client
			.get_ticker_details("DOESN'T EXIST", Some(&params))
			.unwrap_err();
		match resp {
			PolygonError::EmptyResponse() => {}
			e => panic!("bad error type {}", e)
		};
	}
}
