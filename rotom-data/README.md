# Rotom Data
The Rotom Data functions as the core data layer for the other components of the system. It can be thought of as the heart-beat the trading system. This layer needs to be solid as other parts of the system will make decisions based on the data transmitted from this layer.

## Terminology
<h3>Unit Struct</h3>
A struct that contains no fields. They are valueless and only has a impl body associated with it. For instance:
```
pub struct OrderBookL2;
pub struct WebSocketClient;
pub struct Trades
```

## High Level Overview
The system can process various types of MarketEvent within a single unified stream. Streams can be constructed using the builder methods located in the `/src/streams/builder` directory. An example of creating each of these streams can be found in the `/examples` directory.

The system subscribes to a market event in this format:
```
[
    (Exchange, base currency, quote currency, stream kind),
    (Exchange, base currency, quote currency, stream kind),
    (Exchange, base currency, quote currency, stream kind),
]
```

Note:
The approach to subscribing to the single and multi streams differs from the dynamic stream. The dynamic stream utilises enums from `/src/shared/subscription_models.rs` rather than the actual MarketEvent unit struct in the `/src/event_models/market_event.rs` file. This is a workaround to bypass the Rust type system as a `Vec<( _, _, _, _)>` (vec of tuples) require the same types. Hence, we cannot use `(BinanceSpot, "btc", "usdt", OrderBookL2)` and `(PoloniexSpot, "btc", "usdt", Trades)` in the same vec, so we replace it with a enum of the same type. However, further down the process, specifically in the `WebSocketClient::init()` function, these enums and unit structs will be transformed into their corresponding concrete types for the specific exchange via the `StreamSelector`.

## Market Event Process flow
The process flow to subscribe to a web socket stream is depicted below in the format `[ filename, function ]`. If you ever need a refresher on how the data layer operates, please refer to this process flow.
```
1. [dynamic_stream.rs, init()] ->
2. [consumer.rs, consume()] -> 
3. [protocol::ws::mod.rs, WebSocketClient::init()] -> 
4. [poll_next.rs, ExchangeStream::new()]
```

In the `poll_next.rs` is where the events get deserialised and transformered into MarketEvent<T>. These events then travel back in the reverse order of
the depicted flow above, i.e:

```
1. [poll_next.rs, ExchangeStream::new()] -> 
2. [protocol::ws::mod.rs, WebSocketClient::init()] -> 
3. [consumer.rs, consume()] -> 
4. [dynamic_stream.rs, init()]
```

## Types of MaketEvents
Every unit struct in the `/src/event_models` has to implement a SubKind trait, short for subscription kind. The SubKind trait requires an
associative type called `Event`. This associatve type should NOT be a unit type and have the fields for the corresponding unit type. Lets look at the
L2 order book as an example. This can be found in `/src/event_models/event_book.rs`. The unit struct for this is:

All unit structs in the `/src/event_models` directory must implement a SubKind trait, short for subscription kind. The SubKind trait mandates an associated type called Event. This associated type should NOT be a unit type and must contain the fields for the corresponding unit type struct. Let's consider the L2 order book as an example, which can be located in `/src/event_models/event_book.rs`. The unit struct for this is:
```
pub struct OrderBookL2;
```

While the associative type is:
```
pub struct EventOrderBook {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}
```

The unit struct itself does not contain fields, but it includes an associated type that does have the necessary fields. Specifically, the MarketEvent utilises this associated type to encapsulate the most recent book order before broadcasting the event to the entire system. Subscribing to an `OrderBookL2` will produce a `MarketEvent<EventOrderBook>`, while subscribing to a `Trades` stream will generate a `MarketEvent<EventTrade>`. Beneath the surface, the system harnesses Rust's robust type system to transform the unit structs into these MarketEvents.

## Trasformers
The ExchangeStream struct impl the Stream trait from the futures crate. This whole struct becomes a Stream that you can call `.next()` on. The ExchangeStream struct plays a vital role in how the transformers work. The tranformers are essentially structs embedded inside the ExchangeStream struct that houses a transform function to transform the data before buffering it so it can be polled.There are 2 main transfomers to deal with:

1. StatelessTransformer
2. Book

The stateless transformer is used for MarketEvents such as trades because no transformations are desired before emitting the event to the rest of the system. On the other hand, the book transformer plays a crucial role and is not stateless. This transformer initialises the orderbooks with a snapshot (if required), the tick sizes, and then receives updates to updated the local orderbook before emitting the updated orderbook to the rest of the system. The book transformer is more intricate than the statelessTransformer, as the statelessTransformer essentially only function as deserialisation mechanism. While the book transformer deserialises book updates, updates the local orderbook, and then transmits the updated `MarketEvent<EventOrderBook>`.

## Consumer
The "consume" function in the `/src/streams/consumer.rs` directory is crucial for the system. It handles the automatic reconnection of the websocket. This function operates in an infinite loop and continuously reconnects to the websocket if the error received is a terminal error. If it is a none terminal error, the errors are either ignored or logged before polling the next message. Please take a look at the code for a better understanding.

## How to add more exchange?
Adding additional exchanges involves creating a new directory in `/src/exchange` and implementing the `Connector` trait to the new exchange. Additionally, you'll need to provide the concrete types for the StreamSelector related to the new connector. The stream selector represents a combination of the exchange and stream kind. Implementing the stream selectors is crucial as it facilitates the translation of general unit structs from the `/src/event_models` directory into exchange-specific ones. For example, `OrderBookL2` becomes `BinanceSpotBookUpdate` for Binance and `PoloniexSpotBookUpdate` for Poloniex. Specifying these concrete types enables the Rust compiler to compile the code and translate the generic functions to accept these specific types. Again, please have a look at how the other exchange connectors are implemented.

If a specific stream kind does not currently exist, you will need to create it. Please navigate to `/src/model_events` directory and generate a file to represent this new stream kind. For instance, if the OHLCV stream kind has not yet been implemented, go to `/src/event_models/` and create a file named `event_ohlcv.rs`. Then define a unit struct called ohlcv and implement the `SubKind` trait for this unit type. When implementing `Subkind`, name the associative type with "Event" prefix e.g. `EventOhlcv`. You can then use this in the stream selector to specify the concrete type of the OHLCV for the desired exchange. If you still have difficulty understanding, please refer to how other events are implemented.

## Stream Builder types
*Note: Please see the `/examples` as a reference*

<h3>Single</h3>

- Each `.subscribe()` call will create a separate web socket
- This builder can only take the same exchange and stream kind combination in each `.subscribe()` call. For example, this is not allowed:

```
[
    (BinancSpot, "btc", "usdt", Trades),
    (Poloniex, "btc", "usdt", Trades),
]

or

[
    (BinancSpot, "btc", "usdt", Trades),
    (PoloniexSpot, "btc", "usdt", Trades),
]
```

<h3>Multi</h3>

- Each `.subscribe()` call will create a separate web socket
- `.subscribe()` is encased in a `.add()` to be able to add muli exchnanges to a particular trade
- Please see `/examples`

<h3>Dynamic</h3>

- Each `vec[]!` inside the `init()` method will create at least one web socket. The total number of web sockets created depends on the different combinations of `Exchange` and `StreamKind` provided in the `vec![]`. Below are examples of how this works:

One web socket:

```
vec![
    (ExchangeId::PoloniexSpot, "ada", "usdt", StreamKind::L2),
    (ExchangeId::PoloniexSpot, "arb", "usdt", StreamKind::L2),
    (ExchangeId::PoloniexSpot, "eth", "usdt", StreamKind::L2),
    (ExchangeId::PoloniexSpot, "btc", "usdt", StreamKind::L2),
]
```

Two web sockets:

```
vec![
    (ExchangeId::PoloniexSpot, "ada", "usdt", StreamKind::L2),
    (ExchangeId::PoloniexSpot, "arb", "usdt", StreamKind::L2),
    (ExchangeId::BinanceSpot, "eth", "usdt", StreamKind::L2), // New exchange added
    (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::L2), // New exchange added
]
```

Three web sockets:

```
vec![
    (ExchangeId::PoloniexSpot, "ada", "usdt", StreamKind::L2),
    (ExchangeId::PoloniexSpot, "arb", "usdt", StreamKind::L2),
    (ExchangeId::BinanceSpot, "eth", "usdt", StreamKind::L2),
    (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::Trades), // New stream kind added for Binance
]
```

Four web sockets:

```
vec![
    (ExchangeId::PoloniexSpot, "ada", "usdt", StreamKind::L2),
    (ExchangeId::PoloniexSpot, "arb", "usdt", StreamKind::L2), // New stream kind added for Poloniex
    (ExchangeId::BinanceSpot, "eth", "usdt", StreamKind::L2),
    (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::Trades),
]
```

## Important
- When deserializing the symbol for each stream, ensure that the symbol matches the format required for sending orders to buy/sell. For instance, if Poloniex requires symbol in the format `BTC_USDT`, make sure you deserialise the symbol in this format, as the trading sytem will use this later for order submission. If we were to receive symbol in the format `btcusdt` then it will be hard or near impossible to reconcil this back to `BTC_USDT`. 
- Ensure that the keys for the order books in the `MultiBookTransformer`, specifically the `String` in `HashMap<String, InstrumentOrderBook<_>>`, match the symbols being streamed from the WebSocket. The transformer performs lookups based on the streamed symbol, so if it cannot find a matching key, you will encounter a `SocketError::OrderBookFindError`.
- The keys for the order book HashMaps are initialized when implementing the `ExchangeTransformer` for the `MultiBookTransformer`. If you examine the implementation closely, you will see that these keys are retrieved from the `Market` associative type of each exchange `Connector`. Therefore, it is crucial that they match the format of the symbols received from each stream.
- In summary, ensure that the symbol for each stream is formatted the same way as the `Market` associative type for each exchange `Connector`, and ensure this format also matches the keys in the `MultiBookTransformer`. Otherwise, you will encounter an `SocketError::OrderBookFindError` error.
- Also note that each HashMap of orderbooks i.e., `HashMap<String, InstrumentOrderBook<_>>` are for a single exchange. We should not have a Poloniex `BTC_USDT` and a Binance `BTC_USDT` in the same HashMap of orderbooks.

## Thoughts for Future State
- When the `/src/protocol` directory gets more complex i.e., replacing Tokio Tungstenite and Serde with a custom faster solution, move this directory into a separate workspace.

# Todos
- TESTING
- TOP LEVEL SUBSCRIPTION VALIDATOR ie. does binance supppot spot or does poloniex support options. Simple match statement will suffice
- DOUBLE CHECK TICK SIZE BEFORE PRODUCTION
- CUSTOM POLONIEX DESERIALIZERS
- PROCESS CUSTOM PING FOR POLONIEX
- BOOK AND TRADE IN ONE STREAM UPDATE
