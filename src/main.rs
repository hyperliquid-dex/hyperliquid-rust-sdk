use hyperliquid_rust_sdk::InfoClient;

#[tokio::main]
async fn main() {
    let my_info = InfoClient::new(None, None);
    let user_address = "0x010461c14e146ac35fe42271bdc1134ee31c703a";
    let user_state = my_info.user_state(user_address).await;
    println!("{:?}", user_state);
}
