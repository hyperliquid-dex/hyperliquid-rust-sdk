pub mod req; 
pub mod wallet; 
pub mod meta; 
pub mod consts; 
pub mod exchange; 
pub mod signature; 
pub mod info;
// use ethers::signers::Signer; 
// use ethers::{types::TransactionRequest, prelude::LocalWallet};

#[tokio::main]
async fn main() {
    let my_info = info::Info::new(None, None); 
    let user_address = String::from("0x010461c14e146ac35fe42271bdc1134ee31c703a"); 
    let open_orders = my_info.open_orders(user_address).await; 
    println!("{:?}", open_orders); 
}
