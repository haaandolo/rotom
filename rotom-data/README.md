# Rotom Data
The Rotom Data functions as the core data layer for the other components of the system. It can be thought of as the heart-beat the trading system. This layer needs to be solid as other parts of the system will make decisions based on the data transmitted from this layer.

## Terminology
<h5>Unit Struct<h5>
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
The approach to subscribing to the single and multi streams differs from the dynamic stream. The dynamic stream utilises enums from `/src/shared/subscription_models.rs` rather than the actual MarketEvent unit struct in the `/src/event_models/market_event.rs` file. This is a workaround to bypass the Rust type system as a Vec<( _, _)> (vec of tuples) require the same types. Hence, we cannot use `(BinanceSpot, "btc", "usdt", OrderBookL2)` and `(PoloniexSpot, "btc", "usdt", Trades)` in the same vec, so we replace it with a enum of the same type. However, further down the process, specifically in the `WebSocketClient::init()` function, these enums and unit structs will be transformed into their corresponding concrete types for the specific exchange via the `StreamSelector`.

## Market Event Process flow
The process flow to subscribe to a web socket stream is depicted below in the format `[ filename, function ]`. If you ever need a refresher on how the data layer operates, please refer to this process flow.
```
1. [dynamic_stream.rs, init()] -> 2.[consumer.rs, consume()] -> 3.[protocol::ws::mod.rs, WebSocketClient::init()] -> 4.[poll_next.rs, ExchangeStream::new()]
```

In the `poll_next.rs` is where the events get deserialised and transformered into MarketEvent<T>. These events then travel back in the reverse order of
the depicted flow above, i.e:

```
1.[poll_next.rs, ExchangeStream::new()] -> 2.[protocol::ws::mod.rs, WebSocketClient::init()] -> 3.[consumer.rs, consume()] -> 4. [dynamic_stream.rs, init()]
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
The consume function in the /streams directory plays another important part of the system.
This is the place that the websocket get auto-reconnected. How this works is that this is 
infinite loop where it will keep reconnecting to the websocket if the error received for 
the inital disconnection is not a terminal error. Please have a read of the code if you are
unsure.

## How to add more exchange?
Adding more exchanges requires you to create a new directory with exchange connector. To
do this, please implement a Connector trait for it. In addition you will also need to 
provide the concrete types for the StreamSelector for the new connector. The stream selector
is a combination of the exchange and stream kind. Implementing these stream selectors are 
very important as this is when the general unit structs from the /event_models directory
gets translated into the exchange specific ones i.e. OrderBookL2 becomes BinanceSpotBookUpdate 
and for Poloniex it becomes PoloniexSpotBookUpdate. We need to specified these concrete types
so the Rust compiler can compile the code and turn the generic functions to take these 
concrete types.

In the event a stream kind does not exist, you will have to make one. Please go into the 
event model and make a file representing this new stream kind. For example, the OHLCV
stream kind has not been implemented yet. Go to /event_models and make a file called
event_ohlcv.rs. Make a unit struct called ohlcv and impl the SubKind trait for this unit
type. Call the associative type of the unit struct when impl Subkind, EventOhlcv. Then use 
this in the stream selector to provide the concrete type of the Ohlcv of the exchange you want.
If you do not understand still please have a look at how the other events are implemented in
the /event_models directory.

# Todos
- DOUBLE CHECK TICK SIZE BEFORE PRODUCTION
- CUSTOM POLONIEX DESERIALIZERS
- PROCESS CUSTOM PING FOR POLONIEX
- BOOK AND TRADE IN ONE STREAM UPDATE
