use assert_cli::Assert;

#[test]
fn test_version() -> Result<(), std::io::Error> {
    Assert::main_binary()
        .with_args(&["version"])
        .stdout()
        .is(&*format!("srvc {}\n", env!("CARGO_PKG_VERSION")))
        .stderr()
        .is("")
        .unwrap();
    Assert::main_binary()
        .current_dir("test-resources/simple")
        .with_args(&["version"])
        .stdout()
        .is(&*format!("srvc {}\n", env!("CARGO_PKG_VERSION")))
        .stderr()
        .is("")
        .unwrap();
    Ok(())
}
