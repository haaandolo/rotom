### Core Components Required

- Market data: 
This component is able to spin up multiple ws connections and send to require parts of the system. Example may be:

WS1 includes - only one sub as may be a lot of data
.subscribe("binance", "btc", "usdt", "orderbookl2")

WS2 includes - only one sub as may be a lot of data
.subscribe("binance", "eth", "usdt", "trades")

WS3 includes - more subs as smaller data loads
.subscribe("poloniex", "arb", "usdt", "trades")
.subscribe("poloniex", "sushi", "usdt", "trades")
.subscribe("poloniex", "pepe", "usdt", "trades")

We need to normalise this into into something like: 
Events {
    price: float
    quantity: float
    is_buy: bool
    is trade: bool
}

Then send to parts of the systems we need

- Trader:
Takes the events of market event and generates signals. The trader only trades one pair and has its own thread.
It has a many to one relationship with the engine i.e., many traders to one engine. Trader has its own data hander and
strategy.

- Engine
Able to handle an arbitary number of trader for a given market

- Portfolio
aggrgate portfolio of system. 


Note:
- each sub has to be from same exchnage and of same same stream type