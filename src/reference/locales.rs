extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Locale {
  pub locale: String,
  pub name:   String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalesResponse {
  pub results: Vec<Locale>,
  // For debugging
  pub status:  String
}

impl Client {
  pub fn get_locales(&mut self) -> std::io::Result<LocalesResponse> {
    let uri = format!("{}/v2/reference/locales", self.api_uri);

    let resp = self.get_response::<LocalesResponse>(&uri)?;

    Ok(resp)
  }
}

#[cfg(test)]
mod locales {
  use crate::client::Client;

  #[test]
  fn works() {
    let mut client = Client::new();
    let locales = client.get_locales().unwrap();
    assert!(locales.results.len() > 17);
    assert!(locales
      .results
      .iter()
      .any(|res| res.locale == String::from("US")));
  }
}
