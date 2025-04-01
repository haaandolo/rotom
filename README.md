# Rotom

### Spot Arbitrage Scanner for Cryptocurrency Markets
This high-performance implementation monitors real-time arbitrage opportunities across cryptocurrency exchanges. The system concurrently processes over 6,000 data streams from 8 major exchanges, capturing comprehensive market dynamics for all active USDT trading pairs. It is very memory effient using less than 50MB for streaming in all the data.

### Key Features:
- Handles both Level 2 orderbook data and trade execution feeds
- Processes the complete universe of USDT trading pairs
- Scales efficiently across multiple exchange integrations

### Implementation Details
All exchange-specific integration code is organized within the rotom-data/src/exchange directory. Each integration is modular and follows a consistent pattern for data normalization and processing.

### Extensibility
The system is designed for easy expansion. For implementing additional exchanges, please reference the documentation in the rotom-data README, which outlines the integration process and requirements.

### Current Exchanges Integrated
```
1. Ascendex
2. Binance - make account to get API Key
3. Coinex
4. HTX
5. Kucoin
6. Okx - make account to get API Key
7. Phemex - make account to get API Key
8. Woox
9. Poloniex
```

### Running the Spot Arbitrage Scanner
1. Clone the repo
2. Make a account at the exchanges the have a `make account to get API Key` in the above list and paste the api key and secret into the `config-example.toml` file.
3. Change the name of the `config-example.toml` file to `config.toml`. Then make a `.cargo` file in the root directory and place the `config.toml` file in this directory.
4. Then run `cargo run -p rotom-scanner --release`

### Scanner Architecture
The scanner functions as a lightweight Redis-like instance with built-in monitoring capabilities. It exposes an HTTP interface that provides real-time diagnostics, including:

- System health metrics
- Active socket connection counts
- Performance statistics

Built on an Actix web framework foundation, the system leverages Tokio channels for efficient request-response handling. While the current implementation uses a single-producer model optimized for individual user access, this design choice was intentional to maintain simplicity and performance for the primary use case.

### Implementation Details
For a complete understanding of the scanner's operation and API usage patterns, refer to the test suite located at `rotom-scanner/spot_scanner/scanner/mod_test`. These tests demonstrate the full request lifecycle and expected responses.

### Endpoints
After the cargo run we can use these endpoints from postman or a front end web app to get the current top spreads. Please refer to `example_responses` for return responses

Health of scanner: `http://localhost:8080/ws-status`
Get top spreads: `http://localhost:8080/`
Get spread history of specific currency pair: `http://localhost:8080/spread-history?base_exchange=WooxSpot&quote_exchange=KuCoinSpot&base_instrument=kaito&quote_instrument=usdt`