use std::path::PathBuf;

mod common;

#[test]
fn test_simple() -> Result<(), std::io::Error> {
    let dir = "test-resources/simple";
    let sink = PathBuf::from(dir).join("sink.jsonl");
    if sink.exists() {
        std::fs::remove_file(&sink)?;
    };
    common::cmd(200)
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

#[test]
fn test_wrong_name() -> Result<(), std::io::Error> {
    let dir = "test-resources/wrong-name";
    let sink = PathBuf::from(dir).join("sink.jsonl");
    if sink.exists() {
        std::fs::remove_file(&sink)?;
    };
    common::cmd(100)
        .current_dir(dir)
        .args(&["review", "simpel"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("Error: No flow named \"simpel\" in \"sr.yaml\"\n");
    assert_eq!(false, sink.exists());
    Ok(())
}
