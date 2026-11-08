use super::*;
#[test]
fn cli_validates_limits_and_positive_values() {
    for args in [
        vec!["tool"],
        vec!["tool", "--count", "0"],
        vec!["tool", "--duration", "NaN"],
        vec!["tool", "--count", "1", "--rate", "0"],
        vec!["tool", "--count", "1", "--sizes", "0"],
        vec!["tool", "--count", "1", "--duration", "1"],
    ] {
        assert!(Args::try_parse_from(args).is_err());
    }
}
