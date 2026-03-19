//! # Pane-Vortex V2 IPC Client
//!
//! CLI binary for IPC bus interaction. Connects to the daemon's Unix domain
//! socket, performs NDJSON handshake, executes a subcommand, and exits.
//!
//! ## Subcommands
//! - `connect`   — Test bus connectivity (handshake only)
//! - `subscribe` — Stream matching events (persistent)
//! - `submit`    — Submit a task to the bus queue
//! - `cascade`   — Dispatch cascade handoff to another tab
//! - `disconnect` — Graceful disconnection
//!
//! ## Environment
//! - `PANE_VORTEX_SOCKET`  — Unix socket path
//! - `PANE_VORTEX_ID`      — Sphere ID for handshake
//! - `PANE_VORTEX_TIMEOUT` — Operation timeout in seconds (default: 5)

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use pane_vortex::m1_foundation::m01_core_types::PaneId;
use pane_vortex::m7_coordination::m30_bus_types::{BusFrame, BusTask, TaskTarget};

// ── Exit Codes ──

const EXIT_OK: u8 = 0;
const EXIT_CONNECTION: u8 = 1;
const EXIT_HANDSHAKE: u8 = 2;
const EXIT_TIMEOUT: u8 = 3;
const EXIT_PROTOCOL: u8 = 4;

// ── Config ──

struct Config {
    sphere_id: String,
    timeout_secs: u64,
    socket_path: PathBuf,
}

impl Config {
    fn from_env() -> Self {
        let socket_path = std::env::var("PANE_VORTEX_SOCKET")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("XDG_RUNTIME_DIR")
                    .map(|r| PathBuf::from(r).join("pane-vortex-bus.sock"))
            })
            .unwrap_or_else(|_| PathBuf::from("/run/user/1000/pane-vortex-bus.sock"));

        Self {
            sphere_id: std::env::var("PANE_VORTEX_ID")
                .unwrap_or_else(|_| format!("client:{}", std::process::id())),
            timeout_secs: std::env::var("PANE_VORTEX_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            socket_path,
        }
    }
}

// ── Subcommands ──

#[derive(Debug)]
enum Cmd {
    Connect,
    Subscribe { patterns: Vec<String> },
    Submit { description: String, target: String },
    Cascade { target: String, brief: String },
    Disconnect,
}

fn parse_args() -> Option<Cmd> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return None;
    }

    match args[0].as_str() {
        "connect" => Some(Cmd::Connect),
        "subscribe" => {
            let patterns = args[1..].to_vec();
            if patterns.is_empty() {
                Some(Cmd::Subscribe {
                    patterns: vec!["*".to_string()],
                })
            } else {
                Some(Cmd::Subscribe { patterns })
            }
        }
        "submit" => {
            let description = args.get(1).cloned().unwrap_or_else(|| "task".to_string());
            let target = args
                .iter()
                .position(|a| a == "--target")
                .and_then(|i| args.get(i + 1).cloned())
                .unwrap_or_else(|| "any-idle".to_string());
            Some(Cmd::Submit {
                description,
                target,
            })
        }
        "cascade" => {
            let target = args.get(1).cloned().unwrap_or_else(|| "fleet-beta".to_string());
            let brief = args
                .iter()
                .position(|a| a == "--brief")
                .and_then(|i| args.get(i + 1).cloned())
                .unwrap_or_else(|| "cascade task".to_string());
            Some(Cmd::Cascade { target, brief })
        }
        "disconnect" => Some(Cmd::Disconnect),
        _ => {
            print_usage();
            None
        }
    }
}

fn print_usage() {
    eprintln!("Usage: pane-vortex-client <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  connect                      Test bus connectivity");
    eprintln!("  subscribe [patterns...]       Stream matching events");
    eprintln!("  submit <desc> [--target T]    Submit a task");
    eprintln!("  cascade <target> [--brief B]  Dispatch cascade handoff");
    eprintln!("  disconnect                    Graceful disconnection");
    eprintln!();
    eprintln!("Environment:");
    eprintln!("  PANE_VORTEX_ID       Sphere ID (default: client:<pid>)");
    eprintln!("  PANE_VORTEX_SOCKET   Socket path");
    eprintln!("  PANE_VORTEX_TIMEOUT  Timeout seconds (default: 5)");
}

// ── Connection ──

async fn connect_and_handshake(
    config: &Config,
) -> Result<(BufReader<tokio::io::ReadHalf<UnixStream>>, tokio::io::WriteHalf<UnixStream>), u8> {
    let stream = match tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        UnixStream::connect(&config.socket_path),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            eprintln!("connection failed: {e}");
            return Err(EXIT_CONNECTION);
        }
        Err(_) => {
            eprintln!("connection timeout");
            return Err(EXIT_TIMEOUT);
        }
    };

    let (read_half, mut write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);

    // Send handshake
    let handshake = BusFrame::Handshake {
        pane_id: PaneId::new(&config.sphere_id),
        version: "2.0".to_string(),
    };

    let mut line = serde_json::to_string(&handshake).map_err(|_| EXIT_PROTOCOL)?;
    line.push('\n');
    write_half
        .write_all(line.as_bytes())
        .await
        .map_err(|_| EXIT_CONNECTION)?;

    // Read welcome
    let mut response = String::new();
    match tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        reader.read_line(&mut response),
    )
    .await
    {
        Ok(Ok(0)) => {
            eprintln!("connection closed during handshake");
            return Err(EXIT_HANDSHAKE);
        }
        Ok(Ok(_)) => {
            let frame: BusFrame =
                serde_json::from_str(response.trim()).map_err(|_| EXIT_PROTOCOL)?;
            if let BusFrame::Welcome { session_id, .. } = &frame {
                eprintln!("connected as {session_id}");
            } else {
                eprintln!("unexpected handshake response");
                return Err(EXIT_HANDSHAKE);
            }
        }
        Ok(Err(e)) => {
            eprintln!("handshake read error: {e}");
            return Err(EXIT_HANDSHAKE);
        }
        Err(_) => {
            eprintln!("handshake timeout");
            return Err(EXIT_TIMEOUT);
        }
    }

    Ok((reader, write_half))
}

async fn send_frame(
    writer: &mut tokio::io::WriteHalf<UnixStream>,
    frame: &BusFrame,
) -> Result<(), u8> {
    let mut line = serde_json::to_string(frame).map_err(|_| EXIT_PROTOCOL)?;
    line.push('\n');
    writer
        .write_all(line.as_bytes())
        .await
        .map_err(|_| EXIT_CONNECTION)?;
    Ok(())
}

// ── Command execution ──

async fn run_connect(config: &Config) -> u8 {
    match connect_and_handshake(config).await {
        Ok(_) => {
            println!("OK");
            EXIT_OK
        }
        Err(code) => code,
    }
}

async fn run_subscribe(config: &Config, patterns: Vec<String>) -> u8 {
    let (mut reader, mut writer) = match connect_and_handshake(config).await {
        Ok(rw) => rw,
        Err(code) => return code,
    };

    // Send subscribe
    let frame = BusFrame::Subscribe {
        patterns: patterns.clone(),
    };
    if let Err(code) = send_frame(&mut writer, &frame).await {
        return code;
    }

    eprintln!("subscribed to: {}", patterns.join(", "));

    // Stream events
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                eprintln!("connection closed");
                return EXIT_CONNECTION;
            }
            Ok(_) => {
                print!("{line}");
            }
            Err(e) => {
                eprintln!("read error: {e}");
                return EXIT_CONNECTION;
            }
        }
    }
}

async fn run_submit(config: &Config, description: String, target: String) -> u8 {
    let (_reader, mut writer) = match connect_and_handshake(config).await {
        Ok(rw) => rw,
        Err(code) => return code,
    };

    let task_target = match target.as_str() {
        "any-idle" | "any_idle" => TaskTarget::AnyIdle,
        "field-driven" | "field_driven" => TaskTarget::FieldDriven,
        "willing" => TaskTarget::Willing,
        specific => TaskTarget::Specific {
            pane_id: PaneId::new(specific),
        },
    };

    let task = BusTask::new(description, task_target, PaneId::new(&config.sphere_id));
    let frame = BusFrame::Submit { task };
    if let Err(code) = send_frame(&mut writer, &frame).await {
        return code;
    }

    println!("task submitted");
    EXIT_OK
}

async fn run_cascade(config: &Config, target: String, brief: String) -> u8 {
    let (_reader, mut writer) = match connect_and_handshake(config).await {
        Ok(rw) => rw,
        Err(code) => return code,
    };

    let frame = BusFrame::Cascade {
        target: PaneId::new(&target),
        brief,
        source: PaneId::new(&config.sphere_id),
    };
    if let Err(code) = send_frame(&mut writer, &frame).await {
        return code;
    }

    println!("cascade dispatched");
    EXIT_OK
}

async fn run_disconnect(config: &Config) -> u8 {
    let (_reader, mut writer) = match connect_and_handshake(config).await {
        Ok(rw) => rw,
        Err(code) => return code,
    };

    let frame = BusFrame::Disconnect {
        reason: "client requested".to_string(),
    };
    if let Err(code) = send_frame(&mut writer, &frame).await {
        return code;
    }

    println!("disconnected");
    EXIT_OK
}

// ── Main ──

#[tokio::main]
async fn main() -> ExitCode {
    let config = Config::from_env();

    let Some(cmd) = parse_args() else {
        return ExitCode::from(EXIT_PROTOCOL);
    };

    let code = match cmd {
        Cmd::Connect => run_connect(&config).await,
        Cmd::Subscribe { patterns } => run_subscribe(&config, patterns).await,
        Cmd::Submit {
            description,
            target,
        } => run_submit(&config, description, target).await,
        Cmd::Cascade { target, brief } => run_cascade(&config, target, brief).await,
        Cmd::Disconnect => run_disconnect(&config).await,
    };

    ExitCode::from(code)
}
