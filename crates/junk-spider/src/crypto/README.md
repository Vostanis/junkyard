# Blueprint

-------------------------------------------------------------------------

## Binance

RATE_LIMIT = 1200 /60s

tickers = `https://api.binance.com/api/v1/ticker/allBookTickers`

klines = `https://api.binance.com/api/v3/klines`, per symbol

-------------------------------------------------------------------------

## KuCoin

RATE_LIMIT = 4000 /30s

tickers = `https://api.kucoin.com/api/v1/market/allTickers`

klines = `https://api.kucoin.com/api/v1/market/candles?type=1day&symbol=BTC-USDT&startAt=1566703297&endAt=1566789757`, per symbol

-------------------------------------------------------------------------

## MEXC

RATE_LIMIT = 

tickers = `/api/v2/market/ticker`

klines = `/api/v3/klines`
