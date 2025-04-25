use super::responses::*;
use serde_json::from_str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_response_deserialization() {
        let json_str = r#"{
          "channel": "post",
          "data": {
            "id": 9785759392619777167,
            "response": {
              "type": "action",
              "payload": {
                "status": "ok",
                "response": {
                  "type": "order",
                  "data": {
                    "statuses": [
                      {
                        "error": "Price must be divisible by tick size. asset=13"
                      }
                    ]
                  }
                }
              }
            }
          }
        }"#;

        let post_response: PostResponse = from_str(json_str).expect("Failed to deserialize JSON");

        // Verify the deserialization worked correctly
        assert_eq!(post_response.channel, "post");
        assert_eq!(post_response.data.id, 9785759392619777167);
        assert_eq!(post_response.data.response.response_type, "action");
        assert_eq!(post_response.data.response.payload.status, "ok");
        assert_eq!(
            post_response.data.response.payload.response.response_type,
            "order"
        );

        // Verify the error message
        let status = &post_response.data.response.payload.response.data.statuses[0];
        assert_eq!(
            status.error.as_ref().unwrap(),
            "Price must be divisible by tick size. asset=13"
        );
    }
}
