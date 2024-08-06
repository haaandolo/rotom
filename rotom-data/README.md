# Rotom Data
The Rotom data serves as the data layer to the other parts of the system. 
It can be thought of as the heart-beat of the trading system.

## High Level Overview
The system can handle types of market events in one unified stream. Streams 
can be built using the builder methods in the streams/builder directory. There
is an example of how each of these streams can be build in the /examples
directory.

The system subscribes to a market event in this format:
```
[
    (Exchange, base currency, quote currency, stream kind),
    (Exchange, base currency, quote currency, stream kind),
    (Exchange, base currency, quote currency, stream kind),
]
```

Note:
How you subscribe to the single and multi stream in different to the dynamic.
The dynamic one uses enums instead of the actual event unit struct in the 
/event_models directory. This is just to get around the rust type system. Further
down the chain, these enums will get translated into the same event types that
the multi and single streams use.

## Market Event Process flow
When using the builder methods, the stream kind will either be a unit struct from
the /event_model directory, or a enum. As mentioned the enums eventually get
turned into the unit structs in the event model. 

So what exactly is a unit struct. They are structs with no fields in them. For example,
the orderbook unit struct is "pub struct OrderBookL2;". While the trade one is 
"pub struct Trades;". These unit stucts have a "Event" type as an associative type.
These associative types are what the system will use to encase the data from the streams.
For example, if I subscribe to a OrderBookL2, the data the will be received is a
MarketEvent<EventOrderBook>. Where as, if I subscribe to a Trades stream kind then I will
receive a MarketEvent<EventTrade>. Underneath the hood, the system translates the unit
structs into these MarketEvents using Rust powerful type system.

Note:
If you are ever lost on how things happens. Follow these functions step by step to see
what is happening, format = (filename, function):
(dynamic_stream.rs, init()) ->
(consumer.rs, consume()) -> 
(protocol::ws::mod.rs, WebSocketClient::init()) -> 
(poll_next.rs, ExchangeStream::new())

## Trasformers
p.s Since the ExchangeStream struct impl the Stream trait from the futures crate. This whole
struct becomes a Stream that you can call next() on. The ExchangeStream struct plays a vital
role in how the transformers work. The tranformers are essentially structs that have function
associated with them to transform the data and buffering it before it can be polled. There
are 2 main transfomers to deal with:
1. StatelessTransformer
2. Book
The stateless transformer are used for MarketEvents such as trades because we really do not
want to perform and transformations before emitting the event to the rest of the system.
The book transformer is an important. This transformer intialises the orderbooks with a
snapshot (if required) and the tick sizes and receives orderbook updates and updates the
the local orderbook before emitting the update orderbook to the rest of the system. The
book transformer is more complex than the statelessTransformer as the statelessTransformer
one essentially only deserialises the data from the stream. Whereas the book transformer
deserialised book updates, updates the local orderbook before sending update MarketEvents.

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
