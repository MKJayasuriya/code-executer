use axum::routing::get;
use axum::{Router, extract::Json, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(Deserialize)]
struct ExecuteRequest {
    language: String,
    code: String,
}

#[derive(Serialize)]
struct ExecuteResponse {
    output: String,
    error: Option<String>,
}

static CONCURRENCY_LIMIT: usize = 100;
static TIMEOUT_SECONDS: u64 = 5;

type Stats = Arc<Mutex<HashMap<String, usize>>>;

#[tokio::main]
async fn main() {
    let semaphore = Arc::new(Semaphore::new(CONCURRENCY_LIMIT));
    let stats: Stats = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route(
            "/execute",
            post({
                let semaphore = semaphore.clone();
                let stats = stats.clone();
                move |body| execute_handler(body, semaphore.clone(), stats.clone())
            }),
        )
        .route("/", get(handle_home))
        .route(
            "/stats",
            get({
                let stats = stats.clone();
                move || stats_handler(stats.clone())
            }),
        );

    let ip = "0.0.0.0".parse::<Ipv4Addr>().unwrap();
    let addr = SocketAddrV4::new(ip, 3000);

    let url = format!("http://{}", addr);
    println!("Server started in {}", &url);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn execute_handler(
    Json(payload): Json<ExecuteRequest>,
    semaphore: Arc<Semaphore>,
    stats: Stats,
) -> Json<ExecuteResponse> {
    {
        let mut map = stats.lock().unwrap();
        *map.entry(payload.language.to_lowercase()).or_insert(0) += 1;
    }
    let _permit = semaphore.acquire().await.unwrap();

    // Spawn blocking so we don't block async runtime
    let res = tokio::task::spawn_blocking(move || run_code(&payload.language, &payload.code)).await;

    match res {
        Ok(resp) => Json(resp),
        Err(e) => Json(ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Internal error: {e:?}")),
        }),
    }
}

fn run_code(language: &str, code: &str) -> ExecuteResponse {
    match language.to_lowercase().as_str() {
        "python" => run_python(code),
        "javascript" => run_javascript(code),
        "java" => run_java(code),
        "c++" | "cpp" => run_cpp(code),
        other => ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Unsupported language: {other}")),
        },
    }
}

fn run_python(code: &str) -> ExecuteResponse {
    run_script("python3", &["-c", code])
}

fn run_javascript(code: &str) -> ExecuteResponse {
    run_script("node", &["-e", code])
}

fn run_script(cmd: &str, args: &[&str]) -> ExecuteResponse {
    let output = Command::new("timeout")
        .arg(format!("{TIMEOUT_SECONDS}"))
        .arg(cmd)
        .args(args)
        .output();

    match output {
        Ok(out) => ExecuteResponse {
            output: String::from_utf8_lossy(&out.stdout).to_string(),
            error: if out.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&out.stderr).to_string())
            },
        },
        Err(e) => ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Failed to run process: {e}")),
        },
    }
}

fn run_cpp(code: &str) -> ExecuteResponse {
    let dir = tempfile::tempdir().expect("tempdir");
    let uid = Uuid::new_v4().to_string();
    let src_path = dir.path().join(format!("prog_{}.cpp", uid));
    let bin_path = dir.path().join(format!("prog_{}", uid));

    std::fs::write(&src_path, code).expect("write cpp file");

    // Compile
    let compile = Command::new("timeout")
        .arg(format!("{TIMEOUT_SECONDS}"))
        .arg("g++")
        .arg("-o")
        .arg(bin_path.to_str().unwrap())
        .arg(src_path.to_str().unwrap())
        .output();

    if let Err(e) = compile {
        return ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Failed to compile: {e}")),
        };
    }

    let compile_out = compile.unwrap();
    if !compile_out.status.success() {
        return ExecuteResponse {
            output: "".to_string(),
            error: Some(String::from_utf8_lossy(&compile_out.stderr).to_string()),
        };
    }

    // Run
    let run = Command::new("timeout")
        .arg(format!("{TIMEOUT_SECONDS}"))
        .arg(bin_path.to_str().unwrap())
        .output();

    match run {
        Ok(out) => ExecuteResponse {
            output: String::from_utf8_lossy(&out.stdout).to_string(),
            error: if out.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&out.stderr).to_string())
            },
        },
        Err(e) => ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Failed to run binary: {e}")),
        },
    }
}

fn run_java(code: &str) -> ExecuteResponse {
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("Main.java");

    // Write Java code to file (expecting class Main)
    std::fs::write(&file_path, code).expect("write java file");

    // Compile
    let compile = Command::new("timeout")
        .arg(format!("{TIMEOUT_SECONDS}"))
        .arg("javac")
        .arg("Main.java")
        .current_dir(dir.path())
        .output();

    if let Err(e) = compile {
        return ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Failed to compile: {e}")),
        };
    }

    let compile_out = compile.unwrap();
    if !compile_out.status.success() {
        return ExecuteResponse {
            output: "".to_string(),
            error: Some(String::from_utf8_lossy(&compile_out.stderr).to_string()),
        };
    }

    // Run
    let run = Command::new("timeout")
        .arg(format!("{TIMEOUT_SECONDS}"))
        .arg("java")
        .arg("-cp")
        .arg(dir.path())
        .arg("Main")
        .output();

    match run {
        Ok(out) => ExecuteResponse {
            output: String::from_utf8_lossy(&out.stdout).to_string(),
            error: if out.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&out.stderr).to_string())
            },
        },
        Err(e) => ExecuteResponse {
            output: "".to_string(),
            error: Some(format!("Failed to run java: {e}")),
        },
    }
}

#[derive(Serialize)]
struct StatsResponse {
    counts: HashMap<String, usize>,
}

async fn stats_handler(stats: Stats) -> Json<StatsResponse> {
    let map = stats.lock().unwrap();
    Json(StatsResponse {
        counts: map.clone(),
    })
}

pub async fn handle_home() -> &'static str {
    "Welcome to the Code Executor API! Use POST /execute to run code."
}
