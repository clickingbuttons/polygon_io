extern crate serde_json;
extern crate ureq;

use crate::{
  client::Client,
  helpers::make_params,
  with_param
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{de, de::SeqAccess, Deserialize, Serialize};
use std::{
  collections::HashMap,
  fmt,
  io::{self, Error, ErrorKind}
};

// 4 conditions, each 1 byte
// 1: Settlement type
//     @, C,  N,  R,  Y
//     0, 7, 20, 29, 36
// 2: Reason for trade through exempt / other reason
//     F,  O,  O,  4,  5, 6,  7,  8,  9
//    14, 17, 25, 10, 28, 8, 53, 59, 38
// 3: Extended hours / sequence type
//     L,  T,  U,  Z
//    30, 12, 13, 33
// 4: Self regulatory organization trade detail
//     A, B,  D,  E, E, G,  H,  I,  K,  M,  P,  Q,  S, W, X
//     1, 4, 11, 56, 3, 5, 21, 37, 23, 15, 22, 16, 34, 2, 9
//
// Polygon has extra conditions like CAP election, Held, etc.
//
// https://www.utpplan.com/DOC/UtpBinaryOutputSpec.pdf Page 45
// https://www.ctaplan.com/publicdocs/ctaplan/notifications/trader-update/CTS_BINARY_OUTPUT_SPECIFICATION.pdf Page 64
// https://polygon.io/glossary/us/stocks/conditions-indicators has 0-59 (much less than 256)
fn to_conditions<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberArrVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberArrVisitor {
    type Value = u32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("an array of u8")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>
    {
      let mut conditions: [u32; 4] = [
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0)
      ];
      conditions.sort();
      if seq.next_element::<u32>()?.is_some() {
        return Err(de::Error::custom("trades must have 4 or less conditions"));
      }
      let mut res: u32 = 0;
      for (i, c) in conditions.iter().enumerate() {
        if *c > 256 {
          return Err(de::Error::custom(&format!(
            "condition {} should be < 256",
            *c
          )));
        }
        res |= c << (8 * i);
      }
      Ok(res)
    }
  }
  deserializer.deserialize_any(JsonNumberArrVisitor)
}

/* Trade Corrections (NYSE)	
 * Modifier	Indicator
 * 00	Regular trade which was not corrected, changed or signified as cancel or error.
 * 01	Original trade which was late corrected (This record contains the original time - HHMM and the corrected data for the trade).
 * 07	Original trade which was later marked as erroreous
 * 08	Original trade which was later cancelled
 * 10	Cancel record (This record follows '08' records)
 * 11	Error record (This record follows '07' records)
 * 12	Correction record (This record follows'01' records and contains the correction time and the original "incorrect" data). The final correction will be published.
*/
fn to_corrections<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberVisitor {
    type Value = u8;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("an int in [0, 1, 7, 8, 10, 11, 12]")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
          0 => Ok(0),
          1 => Ok(1),
          7 => Ok(2),
          8 => Ok(3),
          10 => Ok(4),
          11 => Ok(5),
          12 => Ok(6),
          c => Err(de::Error::custom(&format!(
            "bad correction {}",
            c
          )))
        }
    }
  }
  deserializer.deserialize_any(JsonNumberVisitor)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
  #[serde(rename(deserialize = "t"))]
  pub ts:         i64,
  // #[serde(rename(deserialize = "y"))]
  // pub ts_participant: Option<i64>,
  // #[serde(rename(deserialize = "f"))]
  // pub ts_trf: Option<i64>,
  #[serde(default)]
  pub symbol:     String,
  #[serde(rename(deserialize = "x"))]
  pub exchange:   u8,
  #[serde(rename(deserialize = "s"))]
  pub size:       u32,
  #[serde(rename(deserialize = "c"), deserialize_with = "to_conditions", default)]
  pub conditions: u32,
  #[serde(rename(deserialize = "p"))]
  pub price:      f64,
  #[serde(rename(deserialize = "z"))]
  pub tape:       u8,
  #[serde(rename(deserialize = "q"))]
  pub seq_id:     u64,
  #[serde(rename(deserialize = "e"), deserialize_with = "to_corrections", default)]
  pub correction: u8
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradesResponse {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  pub results_count: usize,
  pub results: Vec<Trade>,
  // Map is useless
  pub success: bool, // For debugging
  pub db_latency: usize,
  pub uri: Option<String>
}

pub struct TradesParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> TradesParams<'a> {
  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(4)
    }
  }
  
  with_param!(timestamp, i64);
  with_param!(timestamp_limit, i64);
  with_param!(reverse, bool);
  with_param!(limit, usize);
}

const EST_OFFSET: i64 = 5 * 60 * 60 * 1_000_000_000;

impl Client {
  pub fn get_trades(
    &self,
    symbol: &str,
    date: NaiveDate,
    params: Option<&HashMap<&str, String>>
  ) -> io::Result<TradesResponse> {
    let uri = format!(
      "{}/v2/ticks/stocks/trades/{}/{}?apikey={}{}",
      self.api_uri,
      symbol,
      date.format("%Y-%m-%d"),
      self.key,
      match params {
        Some(p) => make_params(p),
        None => String::new()
      }
    );

    let resp = self.get_response(&uri)?;
    let mut resp = resp.into_json::<TradesResponse>()?;
    resp.uri = Some(uri);

    if resp.results.len() == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof, "Results is empty"));
    }

    // Polygon returns the exchange opening time in GMT nanoseconds since epoch
    for row in resp.results.iter_mut() {
      // Only US equities are at "stocks" endpoint
      row.ts -= EST_OFFSET;
      if NaiveDateTime::from_timestamp(row.ts / 1_000_000_000, 0).date() != date {
        return Err(Error::new(
          ErrorKind::BrokenPipe,
          format!(
            "ts {} is out of range for date {}",
            row.ts + EST_OFFSET,
            date.format("%Y-%m-%d")
          )
        ));
      }
      // Add symbol
      row.symbol = resp.symbol.clone();
    }

    Ok(resp)
  }

  // This API should use "q" for paging rather than "ts" which is not unique.
  // This method overcomes that by filtering the "q"s from the last page on the next page.
  pub fn get_all_trades(&self, symbol: &str, date: NaiveDate) -> io::Result<Vec<Trade>> {
    let limit: usize = 50_000;
    let mut params = TradesParams::new().limit(limit);
    let mut res = Vec::<Trade>::new();
    let mut repeated_uids = Vec::<u64>::new();
    loop {
      let page = self.get_trades(symbol, date, Some(&params.params))?.results;
      let page_len = page.len();
      let page_last_ts = page[page_len - 1].ts;
      res.extend(
        page
          .into_iter()
          .filter(|trade| !repeated_uids.contains(&trade.seq_id))
      );
      if page_len != limit {
        break;
      } else {
        repeated_uids = res
          .iter()
          .filter(|trade| trade.ts == page_last_ts)
          .map(|trade| trade.seq_id)
          .collect::<Vec<_>>();
        params = params.timestamp(page_last_ts + EST_OFFSET);
      }
    }

    Ok(res)
  }
}

#[cfg(test)]
mod trades {
  use crate::{client::Client, equities::trades::TradesParams};
  use chrono::NaiveDate;

  #[test]
  fn appl_2004_works() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2004, 01, 02);
    let params = TradesParams::new().params;
    let trades = client.get_trades("AAPL", date, Some(&params)).unwrap();
    let count = 7_452;
    assert_eq!(trades.results_count, count);
    assert_eq!(trades.results.len(), count);
  }

  #[test]
  fn limit_works() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2004, 01, 02);
    let limit = 500;
    let params = TradesParams::new().limit(limit).params;
    let trades = client.get_trades("AAPL", date, Some(&params)).unwrap();
    assert_eq!(trades.results_count, limit);
    assert_eq!(trades.results.len(), limit);
  }

  #[test]
  fn get_all_works() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2020, 01, 02);
    let trades = client.get_all_trades("AAPL", date).unwrap();
    let count = 283_504;
    assert_eq!(trades.len(), count);
  }
}
