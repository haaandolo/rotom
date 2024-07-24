use std::{collections::HashMap, marker::PhantomData};

pub struct MultiBookTransformer<Input, Book> {
    pub orderbooks: HashMap<String, Book>,
    marker: PhantomData<Input>,
}

// Books on exchange likelikey has different sequencing and initialisation methods
// 1. Make exchange specific orderbooks
// 2. Convert current orderbook functions into traits