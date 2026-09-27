# Order response fixtures

`order_create_market_buy.json` and `order_create_market_sell.json` are the
successful EUR/USD market-order examples from
[OANDA's order endpoint documentation](https://developer.oanda.com/rest-live-v20/order-ep/).
The examples date from 2016 and 2018 respectively. Only the account and user
placeholders have been replaced with `"account"` and `1` to make valid JSON.

Keep their omitted fields absent: these fixtures exercise decoding of older
order fills, including fills returned in transaction history.

`trades.json` is the `GET /trades` example from
[OANDA's trade endpoint documentation](https://developer.oanda.com/rest-live-v20/trade-ep/).
Its omitted margin and dividend fields must remain absent.
