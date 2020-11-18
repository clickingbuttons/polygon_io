use ureq::Response;
use std::io::{Error, ErrorKind};

pub fn make_param<T>(param_name: &str, param: Option<T>) -> String
  where
    T: ToString
{
  match param {
    Some(p) => format!("&{}={}", param_name, p.to_string()),
    None => String::new()
  }
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

