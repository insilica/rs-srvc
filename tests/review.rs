mod common;
use common::{delete, file_diff};

use assert_cli::Assert;

#[test]
fn test_simple() -> Result<(), std::io::Error> {
    let dir = "test-resources/simple";
    delete(dir, "sink.jsonl");
    Assert::main_binary()
        .current_dir("test-resources/simple")
        .with_args(&["review", "simple"])
        .stdout()
        .is("")
        .stderr()
        .is("")
        .unwrap();
    assert_eq!(
        (0, String::from("")),
        file_diff(dir, "sink.jsonl", "expected.jsonl")?
    );
    Ok(())
}
