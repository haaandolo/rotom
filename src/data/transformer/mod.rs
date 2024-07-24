pub mod book;
pub mod stateless_transformer;

use serde::Deserialize;

/*----- */
// WebSocket transformer
/*----- */
pub trait Transformer {
    type Error: Send;
    type Input: for<'de> Deserialize<'de>;
    type Output: Send;
    // type OutputIter: IntoIterator<Item = Result<Self::Output, Self::Error>>;
    fn transform(&mut self, input: Self::Input) -> Self::Output;
}
