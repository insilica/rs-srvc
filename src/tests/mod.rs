use super::*;

use std::path::PathBuf;

use lib::Opts;

fn test_opts(cli: &Cli, dir: &str) -> Result<Opts> {
    let err_vec: Vec<u8> = Vec::new();
    let out_vec: Vec<u8> = Vec::new();

    let mut opts = opts(cli)?;
    opts.dir = PathBuf::from(dir);
    opts.err_stream = Box::new(err_vec);
    opts.in_stream = Box::new("".as_bytes());
    opts.out_stream = Box::new(out_vec);
    Ok(opts)
}

fn test_cmd(args: Vec<&str>, dir: &str) -> Result<Opts> {
    let cli = Cli::parse_from(args);
    let mut opts = test_opts(&cli, dir)?;
    run_command(cli, &mut opts)?;
    Ok(opts)
}

fn cmd_out(args: Vec<&str>, dir: &str) -> Result<(String, String)> {
    let opts = test_cmd(args, dir)?;
    let err = opts.err_stream.get_buffer().unwrap();
    let out = opts.out_stream.get_buffer().unwrap();
    Ok((out, err))
}

#[test]
fn test_version() -> Result<()> {
    assert_eq!(
        (format!("srvc {}", VERSION), String::from("")),
        cmd_out(["sr", "version"].into(), ".")?
    );
    Ok(())
}

#[test]
fn test_review_simple() -> Result<()> {
    assert_eq!(
        (String::from(""), String::from("")),
        cmd_out(["sr", "review", "simple"].into(), "test-projects/review-simple")?
    );
    Ok(())
}
