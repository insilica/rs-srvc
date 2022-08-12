mod common;
use common::cmd_out;

#[test]
fn test_version() -> Result<(), std::io::Error> {
    assert_eq!(
        (
            Some(0),
            format!("srvc {}\n", env!("CARGO_PKG_VERSION")),
            String::from("")
        ),
        cmd_out(&vec!["version"], "test-resources/simple")?,
        "version command works",
    );
    Ok(())
}
