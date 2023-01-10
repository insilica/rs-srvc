use std::path::PathBuf;

mod common;

#[cfg(unix)]
#[test]
fn test_label() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 400)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

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
fn test_label_uri() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-uri";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 4000)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
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

#[cfg(unix)]
#[test]
fn test_label_json_schema_url() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-json-schema-url";
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
fn test_generator_order() -> Result<(), std::io::Error> {
    let dir = "test-resources/generator-order";
    common::remove_sink(dir)?;
    common::cmd(400)
        .current_dir(dir)
        .args(&["review", "generator-order"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::check_sink(dir)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_generator_url() -> Result<(), std::io::Error> {
    let dir = "test-resources/generator-url";
    common::remove_sink(dir)?;
    common::cmd(400)
        .current_dir(dir)
        .args(&["review", "generator-url"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::check_sink(dir)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_generator_url_404() -> Result<(), std::io::Error> {
    let dir = "test-resources/generator-url-404";
    common::remove_sink(dir)?;
    common::cmd(400)
        .current_dir(dir)
        .args(&["review", "generator-url"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("Error: Unexpected 404 status for http://127.0.0.1:8877/generator-url/404.jsonl\nError: Step failed with exit code 1\n");
    Ok(())
}

#[test]
fn test_implicit_db() -> Result<(), std::io::Error> {
    let dir = "test-resources/implicit-db";
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
fn test_reviewer_uri_domain() -> Result<(), std::io::Error> {
    let dir = "test-resources/reviewer-uri-domain";
    let sink = PathBuf::from(dir).join("sink.jsonl");
    if sink.exists() {
        std::fs::remove_file(&sink)?;
    };
    common::cmd(200)
        .current_dir(dir)
        .args(&["review", "simple"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("Error: \"reviewer\" is not a valid URI: \"example.com\"\n");
    assert_eq!(false, sink.exists());
    Ok(())
}

#[test]
fn test_reviewer_uri_email() -> Result<(), std::io::Error> {
    let dir = "test-resources/reviewer-uri-email";
    let sink = PathBuf::from(dir).join("sink.jsonl");
    if sink.exists() {
        std::fs::remove_file(&sink)?;
    };
    common::cmd(200)
        .current_dir(dir)
        .args(&["review", "simple"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("Error: \"reviewer\" is not a valid URI: \"user@example.com\"\n  Try \"mailto:user@example.com\"\n");
    assert_eq!(false, sink.exists());
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

#[cfg(unix)]
#[test]
fn test_step_uri() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/step-uri";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 4000)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_flow_uri() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/flow-uri";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 4000)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_base_uri() -> Result<(), rexpect::errors::Error> {
    let dir = "test-resources/base-config";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["review", "label"], 1661192610, 4000)?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}
