use std::{fs, path::PathBuf};

use lib_sr::errors::*;

mod common;

fn test_dir(resource_dir: &str) -> String {
    let mut dir = String::from("test-resources/");
    dir.push_str(resource_dir);
    return dir;
}

/// Test a simple flow that doesn't read from stdin or print to
/// stdout or stderr.
///
/// The flow should output to sink.jsonl. The contents of sink.jsonl
/// are checked against expected.jsonl.
///
/// # Arguments
///
/// * `resource_dir`: A directory in test-resources/ that contains
///     sr.yaml, expected.jsonl, and any other files used by the flow.
/// * `flow_name`: The name of the flow to test, as defined in sr.yaml.
/// * `timeout_millis`: Timeout for the flow to complete, in milliseconds.
///     If the flow takes longer than this, the test will fail and exit.
///     This is ignored when $TEST_SRVC_DISABLE_TIMEOUT is set.
fn test_flow(
    resource_dir: &str,
    flow_name: &str,
    timeout_millis: u64,
) -> lib_sr::errors::Result<()> {
    let dir = test_dir(resource_dir);
    common::remove_sink(&dir)?;
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["flow", flow_name])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::check_sink(&dir)?;
    Ok(())
}

/// Test a simple flow that prints an error message to stderr.
///
/// # Arguments
///
/// * `resource_dir`: A directory in test-resources/ that contains
///     sr.yaml, expected.jsonl, and any other files used by the flow.
/// * `flow_name`: The name of the flow to test, as defined in sr.yaml.
/// * `timeout_millis`: Timeout for the flow to complete, in milliseconds.
///     If the flow takes longer than this, the test will fail and exit.
///     This is ignored when $TEST_SRVC_DISABLE_TIMEOUT is set.
/// * `stderr`: The expected output of stderr.
/// * `sink_should_exist`: Whether the sink file should exist after
///     running the flow.
fn test_flow_err(
    resource_dir: &str,
    flow_name: &str,
    timeout_millis: u64,
    stderr: &'static str,
    sink_should_exist: bool,
) -> Result<()> {
    let dir = test_dir(resource_dir);
    common::remove_sink(&dir)?;
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["flow", flow_name])
        .assert()
        .code(1)
        .stdout("")
        .stderr(stderr);
    assert_eq!(sink_should_exist, common::sink_path(&dir).exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_label() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 400)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("Substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_label_boolean() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-boolean";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 400)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_label_uri() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-uri";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 4000)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("Substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_label_json_schema() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-json-schema";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 400)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("yes")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("no")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("skip")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("yes")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_label_json_schema_url() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/label-json-schema-url";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 400)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("yes")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("no")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("skip")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("yes")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[test]
fn test_simple() -> Result<()> {
    test_flow("simple", "simple", 400)
}

#[test]
fn test_db_override() -> Result<()> {
    let dir = test_dir("test-db-override");
    common::remove_sink(&dir)?;
    common::cmd(400)
        .current_dir(&dir)
        .args(&["flow", "--db", "override.jsonl", "default"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    let db_path = PathBuf::from(&dir).join("override.jsonl");
    let sink_path = common::sink_path(&dir);
    assert_eq!(true, db_path.exists());
    assert_eq!(false, sink_path.exists());
    fs::rename(db_path, sink_path).unwrap();
    common::check_sink(&dir)?;
    Ok(())
}

#[test]
fn test_generator_blank_lines() -> Result<()> {
    test_flow("generator-blank-lines", "generator", 400)
}

#[test]
fn test_generator_order() -> Result<()> {
    test_flow("generator-order", "generator-order", 400)
}

#[test]
fn test_generator_sqlite() -> Result<()> {
    test_flow("generator-sqlite", "default", 5000)
}

#[test]
fn test_generator_order_labels() -> Result<()> {
    test_flow("generator-order-labels", "generator-order-labels", 400)
}

#[test]
fn test_generator_order_labels_existing() -> Result<()> {
    test_flow(
        "generator-order-labels-existing",
        "generator-order-labels-existing",
        400,
    )
}

#[test]
fn test_generator_url() -> Result<()> {
    test_flow("generator-url", "generator-url", 400)
}

#[cfg(unix)]
#[test]
fn test_generator_url_404() -> Result<()> {
    test_flow_err(
        "generator-url-404",
        "generator-url",
        400,
        "Error: Unexpected 404 status for http://127.0.0.1:8877/generator-url/404.jsonl\nError: Step failed with exit code 1\n",
        true
    )
}

#[test]
fn test_implicit_db() -> Result<()> {
    test_flow("implicit-db", "simple", 400)
}

#[test]
fn test_reviewer_uri_domain() -> Result<()> {
    test_flow_err(
        "reviewer-uri-domain",
        "simple",
        400,
        "Error: \"reviewer\" is not a valid URI: \"example.com\"\n",
        false,
    )
}

#[test]
fn test_reviewer_uri_email() -> Result<()> {
    test_flow_err(
        "reviewer-uri-email",
        "simple",
        400,
        "Error: \"reviewer\" is not a valid URI: \"user@example.com\"\n  Try \"mailto:user@example.com\"\n",
        false,
    )
}

#[test]
fn test_wrong_name() -> Result<()> {
    test_flow_err(
        "wrong-name",
        "simpel",
        400,
        "Error: No flow named \"simpel\" in \"sr.yaml\"\n",
        false,
    )
}

#[cfg(unix)]
#[test]
fn test_step_uri() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/step-uri";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 4000)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("Substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_flow_uri() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/flow-uri";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 4000)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("Substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_base_config() -> std::result::Result<(), rexpect::errors::Error> {
    let dir = "test-resources/base-config";
    common::remove_sink(dir).unwrap();
    let mut p = common::spawn(dir, vec!["flow", "label"], 1661192610, 4000)?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_line("y")?;
    p.exp_string("Eye irritation? [Yes/No/Skip]")?;
    p.send_line("n")?;
    p.exp_string("Substance")?;
    p.exp_string("1. \"sodium laureth sulfate\"")?;
    p.exp_string("7. Skip Question")?;
    p.exp_string("?")?;
    p.send_line("1")?;
    p.exp_string("Acute toxicity? [Yes/No/Skip]")?;
    p.send_control('c')?;
    common::check_sink(dir).unwrap();
    Ok(())
}
