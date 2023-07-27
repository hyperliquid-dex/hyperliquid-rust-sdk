use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Coin {
    pub(crate) coin: String,
}

#[derive(Deserialize)]
pub(crate) struct Trades {
    pub(crate) data: Vec<Coin>,
}
