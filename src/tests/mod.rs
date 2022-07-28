use super::*;

use lib::Opts;

fn test_opts(cli: &Cli) -> Opts {
    let err_vec: Vec<u8> = Vec::new();
    let out_vec: Vec<u8> = Vec::new();

    let mut opts = opts(cli);
    opts.err_stream = Box::new(err_vec);
    opts.in_stream = Box::new("".as_bytes());
    opts.out_stream = Box::new(out_vec);
    opts
}

fn test_cmd(args: Vec<&str>) -> Result<Opts> {
    let cli = Cli::parse_from(args);
    let mut opts = test_opts(&cli);
    run_command(cli, &mut opts)?;
    Ok(opts)
}

fn cmd_out(args: Vec<&str>) -> Result<String> {
    Ok(test_cmd(args)?.out_stream.get_buffer().unwrap())
}

#[test]
fn test_version() -> Result<()> {
    assert_eq!(
        format!("srvc {}", VERSION),
        cmd_out(["sr", "version"].into())?
    );
    Ok(())
}
