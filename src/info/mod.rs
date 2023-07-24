mod info_structs; 

use crate::{req::post, consts}; 


pub struct Info {
    client: reqwest::Client, 
    base_url: String, 
}

impl Info {
    pub fn new (optional_client: Option<reqwest::Client>, base_url: Option<String>) -> Self {
        let client = optional_client.unwrap_or_else(|| reqwest::Client::new()); 
        let unwrapped_base_url = base_url.unwrap_or_else(|| consts::MAINNET_API_URL.to_owned()); 

        Info {
            client: client,
            base_url: unwrapped_base_url, 
        }
    }

    pub async fn open_orders (&self, address: String) -> Vec<info_structs::Order> {
        let input = info_structs::OpenOrderInput{
            request_type: String::from("openOrders"), 
            user: address, 
        }; 
        let data = serde_json::to_string(&input).unwrap(); 
        let url = self.base_url.clone() + "/info";
        let return_data = post(&self.client, &url, data).await;
        println!("return data: {}", return_data); 
        let decoded_return_data = serde_json::from_str::<Vec<info_structs::Order>>(&return_data).unwrap(); 
        decoded_return_data
    }

}