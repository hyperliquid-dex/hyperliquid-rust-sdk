use serde::Serialize;

#[derive(Serialize)]
pub struct Signature {
    pub r: String,
    pub s: String,
    pub v: String,
}
