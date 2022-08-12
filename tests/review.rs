mod common;
use common::{cmd_out, delete, file_diff};

#[test]
fn test_simple() -> Result<(), std::io::Error> {
    let dir = "test-resources/simple";
    delete(dir, "sink.jsonl");
    assert_eq!(
        (Some(0), String::from(""), String::from("")),
        cmd_out(&vec!["review", "simple"], dir)?
    );
    assert_eq!(
        (0, String::from("")),
        file_diff(dir, "sink.jsonl", "expected.jsonl")?
    );
    Ok(())
}
