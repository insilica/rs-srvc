use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, LineWriter, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::thread;

use serde::Serialize;
use uuid::Uuid;

use crate::errors::*;

use crate::event::Event;
use crate::lib;
use crate::sr_yaml;

#[derive(Debug)]
pub struct StepProcess {
    process: process::Child,
    step_server: Option<StepServer>,
}

#[derive(Debug)]
pub struct StepServer {
    input_port: u16,
    output_port: u16,
}

fn parse_event(s: &str) -> Result<Event> {
    serde_json::from_str(s).chain_err(|| "Cannot parse event")
}

fn run_step_server(input_listener: TcpListener, output_listener: TcpListener) -> Result<()> {
    let (input, _) = input_listener.accept().chain_err(|| "Listen error")?;
    let (output, _) = output_listener.accept().chain_err(|| "Listen error")?;
    let reader = BufReader::new(input);
    let mut writer = LineWriter::new(output);

    let events = reader
        .lines()
        .map(|line| parse_event(line.chain_err(|| "Failed to read line")?.as_str()));
    for result in events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
        let expected_hash = crate::event::event_hash(event.clone())?;
        let hash = event.hash.clone().unwrap_or("".to_string());
        if hash == "" {
            event.hash = Some(expected_hash);
        } else if expected_hash != hash {
            return Err(format!(
                "Incorrect event hash. Expected: \"{}\". Found: \"{}\".",
                expected_hash, hash
            )
            .into());
        }
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .chain_err(|| "Event serialization failed")?;
        writer.write(b"\n").chain_err(|| "Buffer write failed")?;
    }
    Ok(())
}

fn make_listener(addr: &SocketAddr) -> Result<TcpListener> {
    TcpListener::bind(&addr).chain_err(|| format!("Failed to open TcpListener on {}", addr))
}

fn get_port(listener: &TcpListener) -> Result<u16> {
    Ok(listener
        .local_addr()
        .chain_err(|| "Failed to get local SocketAddr")?
        .port())
}

fn make_step_server() -> Result<StepServer> {
    let addr = SocketAddr::from_str("127.0.0.1:0").chain_err(|| "Failed to create SocketAddr")?;
    let input_listener = make_listener(&addr)?;
    let output_listener = make_listener(&addr)?;
    let input_port = get_port(&input_listener)?;
    let output_port = get_port(&output_listener)?;

    thread::spawn(|| run_step_server(input_listener, output_listener));

    Ok(StepServer {
        input_port,
        output_port,
    })
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

pub fn step_config(config: lib::Config, step: lib::Step) -> Result<lib::Config> {
    let mut labels = Vec::new();
    for label_id in &step.labels {
        let label = config
            .labels
            .get(label_id)
            .ok_or(format!("Label not defined: {}", label_id))?;
        labels.push(label.to_owned());
    }
    Ok(lib::Config {
        current_labels: Some(labels),
        current_step: Some(step),
        ..config
    })
}

#[cfg(unix)]
pub fn get_exe_path() -> Result<PathBuf> {
    Ok(std::env::current_exe()
        .chain_err(|| "Failed to get current exe path")?
        .canonicalize()
        .chain_err(|| "Failed to canonicalize current exe path")?)
}

// std::env::current_exe() returns paths like "\\\\?\\C:\\Users\\"
// We want just the path from C:\ onwards
#[cfg(windows)]
pub fn get_exe_path() -> Result<PathBuf> {
    let handle = windows_win::raw::process::get_current_handle();
    let path = windows_win::raw::process::get_exe_path(handle)
        .chain_err(|| "Failed to get current exe path")?;
    Ok(PathBuf::from(path))
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
    input_port: Option<u16>,
    output: bool,
    exe_path: PathBuf,
) -> Result<StepProcess> {
    let step_config = step_config(config.to_owned(), step.to_owned())?;
    let config_path = make_config(&step_config, dir)?;
    let step_server = if output {
        Some(make_step_server()?)
    } else {
        None
    };
    let (program, args) = get_run_command(step, exe_path)?;
    let sr_input = match input_port {
        Some(port) => format!("127.0.0.1:{}", port.to_string()),
        None => "".into(),
    };
    let sr_output = match &step_server {
        Some(ss) => format!("127.0.0.1:{}", ss.input_port.to_string()),
        None => "".into(),
    };

    let process = process::Command::new(program)
        .args(args)
        .env("SR_CONFIG", config_path)
        .env("SR_INPUT", sr_input)
        .env("SR_OUTPUT", sr_output)
        .spawn()
        .chain_err(|| "Failed to start step sub-process")?;
    Ok(StepProcess {
        step_server,
        process,
    })
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
        let last_ss = last_process.map(|x: StepProcess| x.step_server).flatten();
        let process = run_step(
            config,
            &dir,
            step,
            last_ss.map(|x| x.output_port),
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

pub fn run(opts: &lib::Opts, flow_name: String) -> Result<()> {
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
