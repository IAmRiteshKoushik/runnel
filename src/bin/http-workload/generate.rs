use super::cli::Dataset;
use fake::{
    Fake,
    faker::{
        address::en::StreetName, internet::en::SafeEmail, lorem::en::Sentence,
        name::en::Name,
    },
};
use rand::rngs::StdRng;
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn message(
    rng: &mut StdRng,
    dataset: Dataset,
    size: usize,
    malformed: bool,
    incompatible: bool,
) -> Vec<u8> {
    if malformed {
        return b"{\"request\":".to_vec();
    }
    let email: String = SafeEmail().fake_with_rng(rng);
    let name: String = Name().fake_with_rng(rng);
    let sentence: String = Sentence(3..6).fake_with_rng(rng);
    let (method, path, body, status, response) = match dataset {
        Dataset::Checkout => (
            "POST",
            "/api/orders",
            json!({"email":email,"card":"4111111111111111","items":[{"sku":"SKU-1234","quantity":2}]}),
            201,
            json!({"order_id":sentence,"total":"49.99"}),
        ),
        Dataset::Profile => (
            "PUT",
            "/api/profile",
            json!({"name":name,"email":email,"address":StreetName().fake_with_rng::<String, _>(rng)}),
            200,
            json!({"updated":true}),
        ),
        Dataset::Search => (
            "POST",
            "/api/search",
            json!({"query":sentence,"filters":{"category":"books"}}),
            200,
            json!({"results":[{"id":1,"title":name}]}),
        ),
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let request = if incompatible {
        json!("not an object")
    } else {
        json!({"method":method,"path":path,"headers":{"content-type":"application/json","authorization":format!("Bearer {email}")},"body":body,"remote_ip":"192.0.2.1"})
    };
    let mut record = json!({"timestamp_unix":timestamp,"request":request,"response":{"status":status,"headers":{"content-type":"application/json"},"body":response},"duration_ms":42,"padding":""});
    let mut encoded =
        serde_json::to_vec(&record).expect("JSON value serializes");
    if encoded.len() < size {
        record["padding"] = Value::String("x".repeat(size - encoded.len()));
        encoded = serde_json::to_vec(&record).expect("JSON value serializes");
    }
    encoded
}

#[cfg(test)]
#[path = "../../../tests/unit/loggen_generate.rs"]
mod tests;
