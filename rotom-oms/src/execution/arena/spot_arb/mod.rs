/*
# Execution key points
- 2 versions of execution, buy illiquid & sell illiquid
- always taker buy and sell at the liquid exchange
- always maker buy and sell at bba or deeper at the illiquid exchange

# Buy illiquid
- limit order at bba illiquid exchange
- transfer funds to liquid exchange
- taker out the position

# Sell illiquid
- taker order buy on the liquid exchange
- transfer funds to illiquid exchange
- sell out using limit order at bba in the illiquid exchange
*/

pub mod spot_arb_arena;

