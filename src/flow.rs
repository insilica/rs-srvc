use std::fs::File;
use std::io::{BufReader, BufWriter, LineWriter, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::{env, io};

use log::trace;
use serde::Serialize;
use tempfile::TempDir;
use uuid::Uuid;

use lib_sr::{errors::*, event};
use lib_sr::{Config, Flow, Opts, Step};

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

fn write_str_pretty(v: &impl serde::Serialize) -> Result<String> {
    serde_json::to_string_pretty(v).chain_err(|| "Serialization failed")
}

fn writeln_err(s: &str) -> Result<()> {
    writeln!(io::stderr(), "{}", s).chain_err(|| "Failed to write to stderr")?;
    Ok(())
}

fn run_step_server(input_listener: TcpListener, output_listener: TcpListener) -> Result<()> {
    trace! {"run_step_server"};
    let (input, _) = input_listener.accept().chain_err(|| "Listen error")?;
    let (output, _) = output_listener.accept().chain_err(|| "Listen error")?;
    let reader = BufReader::new(input);
    let mut writer = LineWriter::new(output);

    let events = event::events(reader);
    for result in events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
        let expected_hash = lib_sr::event::event_hash(event.clone())?;
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

    thread::spawn(|| match run_step_server(input_listener, output_listener) {
        Ok(_) => {}
        Err(e) => eprintln!("Error in step server: {:?}", e),
    });

    Ok(StepServer {
        input_port,
        output_port,
    })
}

pub fn make_config(config: &Config, dir: &tempfile::TempDir) -> Result<PathBuf> {
    let mut filename = String::from("config-");
    filename.push_str(&Uuid::new_v4().to_string());
    filename.push_str(".json");
    let path = dir.path().join(filename);
    let file = File::create(&path).chain_err(|| "Failed to create config file for step")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, config).chain_err(|| "Failed to write config for step")?;
    Ok(path)
}

pub fn step_config(config: Config, step: Step) -> Result<Config> {
    let mut labels = Vec::new();
    for label_id in &step.labels {
        let label = config
            .labels
            .get(label_id)
            .ok_or(format!("Label not defined: {}", label_id))?;
        labels.push(label.to_owned());
    }
    Ok(Config {
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

pub fn get_run_command(step: &Step, exe_path: PathBuf) -> Result<(PathBuf, Vec<String>)> {
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
    config: &Config,
    dir: &tempfile::TempDir,
    step: &Step,
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

    let mut cmd = process::Command::new(program);
    cmd.args(args)
        .env("SR_CONFIG", config_path)
        .env("SR_INPUT", sr_input)
        .env("SR_OUTPUT", sr_output)
        .env_remove("SRVC_TOKEN");

    if let Some(env) = &step.env {
        if env.contains(&String::from("SRVC_TOKEN")) {
            if let Ok(token) = env::var("SRVC_TOKEN") {
                cmd.env("SRVC_TOKEN", token);
            }
        }
    }

    match cmd.spawn().chain_err(|| "Failed to start step sub-process") {
        Ok(process) => Ok(StepProcess {
            step_server,
            process: process,
        }),
        Err(e) => {
            writeln_err(&format!("Step failed:\n{}", write_str_pretty(step)?))?;
            Err(e)
        }
    }
}

fn end_steps(processes: Vec<StepProcess>) -> Result<()> {
    let mut error = None;
    for mut process in processes {
        let result = process.process.try_wait();
        match result {
            Ok(Some(status)) => {
                if status.code().is_none() {
                    match process.process.kill() {
                        Ok(_) => {}
                        Err(e) => error = Some(Err(e).chain_err(|| "Failed to kill child process")),
                    }
                }
            }
            Ok(None) => match process.process.kill() {
                Ok(_) => {}
                Err(e) => error = Some(Err(e).chain_err(|| "Failed to kill child process")),
            },
            Err(e) => {
                let _ = process.process.kill();
                error = Some(Err(e).chain_err(|| "Failed to read exit status of child process"))
            }
        }
    }
    match error {
        Some(e) => e,
        None => Ok(()),
    }
}

fn wait_for_steps(mut processes: Vec<StepProcess>) -> Result<()> {
    let mut exit_status = None;
    // Start with a small timeout so small tasks exit quickly,
    // but scale up the timeout to avoid excessive CPU usage in
    // long-running flows.
    let mut timeout = Duration::from_millis(10);
    while processes.len() > 0 && exit_status.is_none() {
        let mut next_processes = Vec::new();
        for mut process in processes {
            thread::sleep(timeout);
            match process.process.try_wait() {
                Ok(Some(status)) => {
                    if status.code() != Some(0) {
                        exit_status = Some(status);
                    }
                }
                Ok(None) => next_processes.push(process),
                Err(e) => return Err(e).chain_err(|| "Error waiting for child process"),
            }
        }
        processes = next_processes;
        if timeout < Duration::from_millis(500) {
            timeout *= 2;
        }
    }

    match exit_status {
        Some(status) => {
            end_steps(processes)?;
            Err(format!(
                "Step failed with exit code {}",
                status
                    .code()
                    .map(|i| i.to_string())
                    .unwrap_or(String::from("None"))
            )
            .into())
        }
        None => Ok(()),
    }
}

fn run_flow_in_dir(flow: &Flow, config: &Config, dir: &TempDir) -> Result<()> {
    if flow.steps.is_empty() {
        return Err("No steps in flow".into());
    }

    let mut steps = Vec::new();
    let flow_steps = &flow.steps.clone();

    for source in &config.sources {
        steps.push(&source.step);
    }
    steps.extend(flow_steps);

    let exe_path = get_exe_path()?;
    let last_step = &flow.steps.last();
    let mut processes = Vec::new();

    for step in steps {
        let is_last_step = last_step.filter(|x| x.to_owned() == step).is_some();
        let last_ss = processes
            .last()
            .map(|x: &StepProcess| x.step_server.as_ref())
            .flatten();
        match run_step(
            config,
            &dir,
            &step,
            last_ss.map(|x| x.output_port),
            !is_last_step,
            exe_path.clone(),
        ) {
            Ok(process) => processes.push(process),
            Err(e) => {
                end_steps(processes)?;
                return Err(e);
            }
        }
    }

    wait_for_steps(processes)
}

pub fn run_flow(flow: &Flow, config: &Config) -> Result<()> {
    let dir = tempfile::Builder::new()
        .prefix("srvc-")
        .tempdir()
        .chain_err(|| "Failed to create temporary directory")?;
    let result = run_flow_in_dir(flow, config, &dir);
    dir.close()
        .chain_err(|| "Failed to delete temporary directory")?;
    return result;
}

pub fn run(
    opts: &mut Opts,
    db: Option<String>,
    flow_name: String,
    reviewer: Option<String>,
) -> Result<()> {
    let yaml_config = sr_yaml::get_config(PathBuf::from(&opts.config))?;
    let mut config = sr_yaml::parse_config(yaml_config)?;
    config.db = db.unwrap_or(config.db);

    let reviewer = match reviewer {
        Some(s) => s,
        None => config.reviewer.ok_or("\"reviewer\" not set in config")?,
    };
    sr_yaml::validate_reviewer(&reviewer)?;
    config.reviewer = Some(reviewer);

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
