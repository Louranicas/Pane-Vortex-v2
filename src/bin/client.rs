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
    Poll,
    Claim { task_id: String },
    Complete { task_id: String },
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
            let target_pos = args.iter().position(|a| a == "--target");
            let target = target_pos
                .and_then(|i| args.get(i + 1).cloned())
                .unwrap_or_else(|| "any-idle".to_string());
            // Description is the first positional arg that isn't --target or its value
            let description = args[1..]
                .iter()
                .enumerate()
                .filter(|(i, a)| {
                    *a != "--target"
                        && target_pos.map_or(true, |tp| *i + 1 != tp + 1)
                })
                .map(|(_, a)| a.as_str())
                .next()
                .unwrap_or("task")
                .to_string();
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
        "poll" => Some(Cmd::Poll),
        "claim" => {
            let task_id = args.get(1).cloned().unwrap_or_default();
            if task_id.is_empty() {
                eprintln!("claim requires a task_id argument");
                return None;
            }
            Some(Cmd::Claim { task_id })
        }
        "complete" => {
            let task_id = args.get(1).cloned().unwrap_or_default();
            if task_id.is_empty() {
                eprintln!("complete requires a task_id argument");
                return None;
            }
            Some(Cmd::Complete { task_id })
        }
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
    eprintln!("  poll                          List pending tasks (HTTP)");
    eprintln!("  claim <task_id>               Claim a pending task (HTTP)");
    eprintln!("  complete <task_id>            Complete a claimed task (HTTP)");
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
        Cmd::Poll => run_http_poll().await,
        Cmd::Claim { task_id } => run_http_claim(&config, task_id).await,
        Cmd::Complete { task_id } => run_http_complete(task_id).await,
    };

    ExitCode::from(code)
}

// ── HTTP-based commands (raw TCP, no hyper) ──

/// Poll for pending tasks via HTTP GET `/bus/tasks`.
async fn run_http_poll() -> u8 {
    let url = pv_url();
    let Ok(resp) = http_get(&format!("{url}/bus/tasks")).await else {
        return EXIT_CONNECTION;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&resp) else {
        eprintln!("failed to parse response");
        return EXIT_PROTOCOL;
    };
    let pending: Vec<&serde_json::Value> = parsed["tasks"]
        .as_array()
        .map_or_else(Vec::new, |arr| {
            arr.iter()
                .filter(|t| t["status"].as_str() == Some("Pending"))
                .collect()
        });
    println!("{}", serde_json::json!(pending));
    EXIT_OK
}

/// Claim a pending task via HTTP POST `/bus/tasks/{id}/claim`.
async fn run_http_claim(config: &Config, task_id: String) -> u8 {
    let url = pv_url();
    let payload = serde_json::json!({"claimer": config.sphere_id});
    let Ok(resp) = http_post(
        &format!("{url}/bus/claim/{task_id}"),
        &payload.to_string(),
    )
    .await
    else {
        return EXIT_CONNECTION;
    };
    println!("{resp}");
    EXIT_OK
}

/// Complete a claimed task via HTTP POST `/bus/tasks/{id}/complete`.
async fn run_http_complete(task_id: String) -> u8 {
    let url = pv_url();
    let Ok(resp) = http_post(&format!("{url}/bus/complete/{task_id}"), "{}").await else {
        return EXIT_CONNECTION;
    };
    println!("{resp}");
    EXIT_OK
}

/// Get PV daemon URL from env.
fn pv_url() -> String {
    std::env::var("PANE_VORTEX_URL").unwrap_or_else(|_| "http://localhost:8132".to_string())
}

/// Raw TCP HTTP GET (fire-and-forget, no hyper).
async fn http_get(url: &str) -> Result<String, u8> {
    let (host, path) = parse_url(url);
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    raw_http(&host, &request).await
}

/// Raw TCP HTTP POST with JSON body (fire-and-forget, no hyper).
async fn http_post(url: &str, body: &str) -> Result<String, u8> {
    let (host, path) = parse_url(url);
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    raw_http(&host, &request).await
}

/// Parse URL into (host:port, path).
fn parse_url(url: &str) -> (String, String) {
    let stripped = url.strip_prefix("http://").unwrap_or(url);
    stripped
        .split_once('/')
        .map_or_else(
            || (stripped.to_string(), "/".to_string()),
            |(h, p)| (h.to_string(), format!("/{p}")),
        )
}

/// Send raw HTTP request via TCP and return response body.
async fn raw_http(host: &str, request: &str) -> Result<String, u8> {
    use tokio::net::TcpStream;
    let Ok(Ok(stream)) = tokio::time::timeout(
        Duration::from_secs(2),
        TcpStream::connect(host),
    )
    .await
    else {
        eprintln!("HTTP connection failed to {host}");
        return Err(EXIT_CONNECTION);
    };
    let (reader, mut writer) = tokio::io::split(stream);
    writer
        .write_all(request.as_bytes())
        .await
        .map_err(|_| EXIT_PROTOCOL)?;
    writer.flush().await.map_err(|_| EXIT_PROTOCOL)?;
    let mut buf_reader = BufReader::new(reader);
    let mut response = String::new();
    loop {
        let mut line = String::new();
        match buf_reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => response.push_str(&line),
        }
    }
    // Extract body after \r\n\r\n
    Ok(response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string())
}
