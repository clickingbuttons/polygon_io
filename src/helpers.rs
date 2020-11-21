use ureq::Response;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

pub fn make_params(params: &HashMap<&str, String>) -> String {
  params.iter().map(|(key, val)| format!("&{}={}", key, val)).collect::<Vec<String>>().join("")
}

pub fn get_response(uri: &str) -> std::io::Result<Response> {
  let resp = ureq::get(&uri)
    .timeout_connect(10_000)
    .timeout_read(10_000)
    .call();

  let status = resp.status();
  if status != 200 {
    return Err(Error::new(
      ErrorKind::NotConnected,
      format!("Server returned {}", status)
    ));
  }
  Ok(resp)
}

