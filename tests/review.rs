mod common;

#[test]
fn test_simple() -> Result<(), std::io::Error> {
    let dir = "test-resources/simple";
    common::delete(dir, "sink.jsonl");
    common::cmd()
        .current_dir(dir)
        .args(&["review", "simple"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert_eq!(
        (0, String::from("")),
        common::file_diff(dir, "sink.jsonl", "expected.jsonl")?
    );
    Ok(())
}
