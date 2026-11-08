use super::*;
use rand::SeedableRng;
#[test]
fn datasets_have_http_details_and_requested_size() {
    let mut rng = StdRng::seed_from_u64(1);
    for dataset in [Dataset::Checkout, Dataset::Profile, Dataset::Search] {
        let bytes = message(&mut rng, dataset, 8192, false, false);
        assert!(bytes.len() >= 8192);
        let event: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(event["request"]["headers"].is_object());
        assert!(event["request"]["body"].is_object());
        assert!(event["response"]["headers"].is_object());
        assert!(event["response"]["body"].is_object());
    }
}

#[test]
fn invalid_cases_are_distinct() {
    let mut rng = StdRng::seed_from_u64(1);
    assert!(
        serde_json::from_slice::<Value>(&message(
            &mut rng,
            Dataset::Checkout,
            1024,
            true,
            true
        ))
        .is_err()
    );
    let event: Value = serde_json::from_slice(&message(
        &mut rng,
        Dataset::Checkout,
        1024,
        false,
        true,
    ))
    .unwrap();
    assert!(event["request"].is_string());
}
