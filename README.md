# polygon-io

Rust [Polygon.io](https://polygon.io) client that verifies timestamps are in queried ranges and uses nanoseconds. Built on ureq and chrono.


## Endpoints
- [ ] /v2/reference/tickers
- [ ] /v2/reference/types
- [ ] /v1/meta/symbols/{symbol}/company
- [ ] /v1/meta/symbols/{symbol}/news
- [ ] /v2/reference/markets
- [ ] /v2/reference/locales
- [ ] /v2/reference/splits/{symbol}
- [ ] /v2/reference/dividends/{symbol}
- [ ] /v2/reference/financials/{symbol}
- [ ] /v1/marketstatus/now
- [ ] /v1/marketstatus/upcoming

### shared?
- [ ] /v2/aggs/ticker/{ticker}/prev
- [x] /v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from}/{to}
- [x] /v2/aggs/grouped/locale/{locale}/market/{market}/{date}

- [ ] /v1/meta/exchanges
- [ ] /v2/ticks/stocks/trades/{ticker}/{date}
- [ ] /v2/ticks/stocks/nbbo/{ticker}/{date}
- [ ] /v1/last/stocks/{symbol}
- [ ] /v1/last_quote/stocks/{symbol}
- [ ] /v1/open-close/{symbol}/{date}
- [ ] /v1/meta/conditions/{ticktype}
- [ ] /v2/snapshot/locale/us/markets/stocks/tickers
- [ ] /v2/snapshot/locale/us/markets/stocks/tickers/{ticker}
- [ ] /v2/snapshot/locale/us/markets/stocks/{direction}

- [ ] /v1/historic/forex/{from}/{to}/{date}
- [ ] /v1/conversion/{from}/{to}
- [ ] /v1/last_quote/currencies/{from}/{to}
- [ ] /v2/snapshot/locale/global/markets/forex/tickers
- [ ] /v2/snapshot/locale/global/markets/forex/{direction}

- [ ] /v1/meta/crypto-exchanges
- [ ] /v1/last/crypto/{from}/{to}
- [ ] /v1/open-close/crypto/{from}/{to}/{date}
- [ ] /v1/historic/crypto/{from}/{to}/{date}
- [ ] /v2/snapshot/locale/global/markets/crypto/tickers
- [ ] /v2/snapshot/locale/global/markets/crypto/tickers/{ticker}
- [ ] /v2/snapshot/locale/global/markets/crypto/tickers/{ticker}/book
- [ ] /v2/snapshot/locale/global/markets/crypto/{direction}
