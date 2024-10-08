use crate::game_rules::resources::Score;

#[test]
fn test_score_display() {
    assert_eq!(Score(0).to_string(), "0");
    assert_eq!(Score(3).to_string(), "3");
    assert_eq!(Score(9).to_string(), "9");
    assert_eq!(Score(33).to_string(), "33");
    assert_eq!(Score(128).to_string(), "128");
    assert_eq!(Score(1000).to_string(), "1,000");
    assert_eq!(Score(9999).to_string(), "9,999");
    assert_eq!(Score(99999).to_string(), "99,999");
    assert_eq!(Score(999999).to_string(), "999,999");
    assert_eq!(Score(1000000).to_string(), "1,000,000");
}
