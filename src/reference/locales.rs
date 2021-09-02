extern crate serde_json;
extern crate ureq;

use crate::{client::Client, helpers::get_response};
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
  pub fn get_locales(&self) -> std::io::Result<LocalesResponse> {
    let uri = format!("{}/v2/reference/locales?apikey={}", self.api_uri, self.key);

    let resp = get_response(&self.agent.agent, &uri)?;
    let resp = resp.into_json::<LocalesResponse>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod locales {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let locales = client.get_locales().unwrap();
    assert!(locales.results.len() > 17);
    assert!(locales
      .results
      .iter()
      .any(|res| res.locale == String::from("US")));
  }
}
