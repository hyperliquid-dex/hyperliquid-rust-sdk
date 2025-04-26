use super::responses::*;
use serde_json::from_str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_response_deserialization() {
        // Test error case
        let error_json_str = r#"{
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

        let error_response: PostResponse =
            from_str(error_json_str).expect("Failed to deserialize error JSON");

        // Verify the error case deserialization
        assert_eq!(error_response.channel, "post");
        assert_eq!(error_response.data.id, 9785759392619777167);
        assert_eq!(error_response.data.response.response_type, "action");
        assert_eq!(error_response.data.response.payload.status, "ok");
        assert_eq!(
            error_response.data.response.payload.response.response_type,
            "order"
        );

        // Verify the error message
        let error_status = &error_response.data.response.payload.response.data.statuses[0];
        assert_eq!(
            error_status.error.as_ref().unwrap(),
            "Price must be divisible by tick size. asset=13"
        );
        assert!(error_status.filled.is_none());

        // Test success case
        let success_json_str = r#"{
          "channel": "post",
          "data": {
            "id": 17312182961762094448,
            "response": {
              "type": "action",
              "payload": {
                "status": "ok",
                "response": {
                  "type": "order",
                  "data": {
                    "statuses": [
                      {
                        "filled": {
                          "totalSz": "11.0",
                          "avgPx": "17.826",
                          "oid": 89150510850
                        }
                      }
                    ]
                  }
                }
              }
            }
          }
        }"#;

        let success_response: PostResponse =
            from_str(success_json_str).expect("Failed to deserialize success JSON");

        // Verify the success case deserialization
        assert_eq!(success_response.channel, "post");
        assert_eq!(success_response.data.id, 17312182961762094448);
        assert_eq!(success_response.data.response.response_type, "action");
        assert_eq!(success_response.data.response.payload.status, "ok");
        assert_eq!(
            success_response
                .data
                .response
                .payload
                .response
                .response_type,
            "order"
        );

        // Verify the filled status
        let success_status = &success_response
            .data
            .response
            .payload
            .response
            .data
            .statuses[0];
        assert!(success_status.error.is_none());
        let filled = success_status.filled.as_ref().unwrap();
        assert_eq!(filled.total_sz, "11.0");
        assert_eq!(filled.avg_px, "17.826");
        assert_eq!(filled.oid, 89150510850);
    }
}
