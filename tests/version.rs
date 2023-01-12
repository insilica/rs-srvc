mod common;

#[test]
fn test_version() -> Result<(), std::io::Error> {
    let mut expected = String::from(format!("srvc {}\n", env!("CARGO_PKG_VERSION")));
    match option_env!("SELF_REV") {
        Some(rev) => expected.push_str(&format!("Revision {}\n", rev)),
        _ => {}
    }

    common::cmd(100)
        .args(&["version"])
        .assert()
        .success()
        .stdout(expected)
        .stderr("");
    Ok(())
}
