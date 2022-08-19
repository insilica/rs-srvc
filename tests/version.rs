mod common;

#[test]
fn test_version() -> Result<(), std::io::Error> {
    common::cmd()
        .args(&["version"])
        .assert()
        .success()
        .stdout(format!("srvc {}\n", env!("CARGO_PKG_VERSION")))
        .stderr("");
    Ok(())
}
