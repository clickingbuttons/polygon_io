extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Types {
	pub types:       HashMap<String, String>,
	pub index_types: HashMap<String, String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypesResponse {
	pub results: Types,
	// For debugging
	pub status:  String
}

impl Client {
	pub fn get_types(&mut self) -> std::io::Result<TypesResponse> {
		let uri = format!("{}/v2/reference/types", self.api_uri);

		let resp = self.get_response::<TypesResponse>(&uri)?;

		Ok(resp)
	}
}

#[cfg(test)]
mod types {
	use crate::client::Client;

	#[test]
	fn works() {
		let mut client = Client::new().unwrap();
		let types = client.get_types().unwrap();
		assert_eq!(
			types.results.types.get("CS").unwrap(),
			&String::from("Common Stock")
		);
	}
}
