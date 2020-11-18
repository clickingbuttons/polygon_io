extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use crate::helpers::get_response;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Types {
  pub types: HashMap<String, String>,
  pub index_types: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypesResponse {
  pub results: Types,
  // For debugging
  pub status: String,
}

impl Client {
  pub fn get_types(&self) -> std::io::Result<TypesResponse> {
    let uri = format!(
      "{}/v2/reference/types?apikey={}",
      self.api_uri,
      self.key
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<TypesResponse>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod types {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let types = client
      .get_types()
      .unwrap();
    assert_eq!(types.results.types.get("CS").unwrap(), &String::from("Common Stock"));
  }
}

