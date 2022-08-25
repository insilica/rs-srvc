use std::path::PathBuf;

mod common;

#[cfg(unix)]
#[test]
fn test_label_boolean() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-boolean";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 400)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_label_json_schema() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-json-schema";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 400)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("yes")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("no")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("skip")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("yes")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[test]
fn test_simple() -> Result<(), std::io::Error> {
    let dir = "test-resources/simple";
    common::remove_sink(dir)?;
    common::cmd(400)
        .current_dir(dir)
        .args(&["review", "simple"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::check_sink(dir)?;
    Ok(())
}

#[test]
fn test_wrong_name() -> Result<(), std::io::Error> {
    let dir = "test-resources/wrong-name";
    let sink = PathBuf::from(dir).join("sink.jsonl");
    if sink.exists() {
        std::fs::remove_file(&sink)?;
    };
    common::cmd(200)
        .current_dir(dir)
        .args(&["review", "simpel"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("Error: No flow named \"simpel\" in \"sr.yaml\"\n");
    assert_eq!(false, sink.exists());
    Ok(())
}
