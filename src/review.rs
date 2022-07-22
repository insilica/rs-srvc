use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process;

use nix::sys::stat;
use nix::unistd;
use uuid::Uuid;

use crate::errors::*;

use crate::lib;
use crate::sr_yaml;
pub struct StepProcess {
    output: Option<PathBuf>,
    process: process::Child,
}

pub fn make_config(config: &lib::Config, dir: &tempfile::TempDir) -> Result<PathBuf> {
    let mut filename = String::from("config-");
    filename.push_str(&Uuid::new_v4().to_string());
    filename.push_str(".json");
    let path = dir.path().join(filename);
    let file = File::create(&path).chain_err(|| "Failed to create config file for step")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, config).chain_err(|| "Failed to write config for step")?;
    Ok(path)
}

pub fn make_fifo(dir: &tempfile::TempDir) -> Result<PathBuf> {
    let mut filename = String::from("fifo-");
    filename.push_str(&Uuid::new_v4().to_string());
    let path = dir.path().join(filename);
    unistd::mkfifo(&path, stat::Mode::S_IRUSR | stat::Mode::S_IWUSR)
        .chain_err(|| "Failed to create named pipe (mkfifo)")?;
    Ok(path)
}

pub fn step_config(config: lib::Config, step: lib::Step) -> Result<lib::Config> {
    Ok(lib::Config {
        current_labels: None,
        current_step: Some(step),
        ..config
    })
}

pub fn get_exe_path() -> Result<PathBuf> {
    Ok(std::env::current_exe()
        .chain_err(|| "Failed to get current exe path")?
        .canonicalize()
        .chain_err(|| "Failed to canonicalize current exe path")?)
}

pub fn get_run_command(step: &lib::Step, exe_path: PathBuf) -> Result<(PathBuf, Vec<String>)> {
    Ok(match step.run_embedded.clone() {
        Some(embedded) => {
            let mut runcmd = "run-embedded-step ".to_string();
            runcmd.push_str(&embedded);
            let args = shell_words::split(&runcmd)
                .chain_err(|| format!("Failed to parse run_embedded command: {}", embedded))?;
            (exe_path, args)
        }
        None => {
            let runcmd = step.run.as_ref().ok_or("Step has no run phase")?.to_owned();
            let args = shell_words::split(&runcmd)
                .chain_err(|| format!("Failed to parse run command: {}", runcmd))?;
            let program = args.first().ok_or("No command to run")?;
            (program.into(), Vec::from(&args[1..]))
        }
    })
}

pub fn run_step(
    config: &lib::Config,
    dir: &tempfile::TempDir,
    step: &lib::Step,
    input: Option<PathBuf>,
    output: bool,
    exe_path: PathBuf,
) -> Result<StepProcess> {
    let step_config = step_config(config.to_owned(), step.to_owned())?;
    let config_path = make_config(&step_config, dir)?;
    let output = if output { Some(make_fifo(dir)) } else { None }.transpose()?;
    let empty_path = PathBuf::new();
    let (program, args) = get_run_command(step, exe_path)?;
    let process = process::Command::new(program)
        .args(args)
        .env("SR_CONFIG", config_path)
        .env(
            "SR_INPUT",
            match input {
                Some(path) => path,
                None => PathBuf::new(),
            },
        )
        .env(
            "SR_OUTPUT",
            match &output {
                Some(path) => path,
                None => &empty_path,
            },
        )
        .spawn()
        .chain_err(|| "Failed to start step sub-process")?;
    Ok(StepProcess { output, process })
}

pub fn run_flow(flow: &lib::Flow, config: &lib::Config) -> Result<process::ExitStatus> {
    let dir = tempfile::Builder::new()
        .prefix("srvc-")
        .tempdir()
        .chain_err(|| "Failed to create temporary directory")?;
    let exe_path = get_exe_path()?;

    let last_step = &flow.steps.last();
    let mut last_process = None;
    for step in &flow.steps {
        let is_last_step = last_step.filter(|x| x.to_owned() == step).is_some();
        let last_output = last_process
            .map(|x: StepProcess| x.output.ok_or("None"))
            .transpose()?;
        let process = run_step(
            config,
            &dir,
            step,
            last_output,
            !is_last_step,
            exe_path.clone(),
        )?;
        last_process = Some(process);
    }
    let process = last_process
        .ok_or("No final step in flow")?
        .process
        .wait()
        .chain_err(|| "Error waiting for child process")?;
    dir.close()
        .chain_err(|| "Failed to delete temporary directory")?;
    Ok(process)
}

pub fn run(opts: lib::Opts, flow_name: String) -> Result<()> {
    let yaml_config = sr_yaml::get_config(PathBuf::from(&opts.config))?;
    let config = sr_yaml::parse_config(yaml_config)?;
    let flow = config.flows.get(&flow_name);
    let flow = match flow {
        Some(flow) => Ok(flow),
        None => Err(format!(
            "No flow named \"{}\" in \"{}\"",
            flow_name, &opts.config
        )),
    }?;
    run_flow(flow, &config)?;
    Ok(())
}
