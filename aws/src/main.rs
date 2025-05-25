use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    net::{Ipv4Addr, SocketAddrV4},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpListener, process::Command, time::timeout};
use uuid::Uuid;

#[derive(Deserialize, Default)]
struct ExecuteRequest {
    language: String,
    code: String,
}

#[derive(Serialize, Deserialize)]
struct ExecuteResponse {
    stdout: String,
    stderr: String,
    status: i32,
}

#[derive(Serialize, Deserialize)]
struct StatsResponse {
    counts: HashMap<String, usize>,
}

type Stats = Arc<Mutex<HashMap<String, usize>>>;

#[derive(Clone)]
struct AppState {
    stats: Stats,
}

async fn stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    let map = state.stats.lock().unwrap();
    Json(StatsResponse {
        counts: map.clone(),
    })
}

fn write_temp_code(code: &str, ext: &str) -> Result<PathBuf, (StatusCode, String)> {
    let tmp_dir = "/tmp/codeexec";
    fs::create_dir_all(tmp_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create temp directory".to_string(),
        )
    })?;

    fs::set_permissions(tmp_dir, fs::Permissions::from_mode(0o755)).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to set temp directory permissions".to_string(),
        )
    })?;

    let filename = format!("code_{}.{}", Uuid::new_v4(), ext);
    let filepath = PathBuf::from(format!("{}/{}", tmp_dir, filename));

    let mut file = fs::File::create(&filepath).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create code file".to_string(),
        )
    })?;

    file.write_all(code.as_bytes()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to write code to file".to_string(),
        )
    })?;

    fs::set_permissions(&filepath, fs::Permissions::from_mode(0o644)).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to set code file permissions".to_string(),
        )
    })?;

    Ok(filepath)
}

async fn execute_code(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteRequest>,
) -> impl IntoResponse {
    let (script_path, ext) = match payload.language.to_lowercase().as_str() {
        "python" => ("/opt/runners/python_runner.sh", "py"),
        "javascript" => ("/opt/runners/js_runner.sh", "js"),
        "java" => ("/opt/runners/java_runner.sh", "java"),
        "cpp" => ("/opt/runners/cpp_runner.sh", "cpp"),
        _ => {
            return (StatusCode::BAD_REQUEST, "Unsupported language".to_string()).into_response();
        }
    };

    let code_path = match write_temp_code(&payload.code, ext) {
        Ok(path) => path,
        Err(resp) => return resp.into_response(),
    };

    if let Err(_) = fs::set_permissions(&code_path, fs::Permissions::from_mode(0o644)) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to set permissions on temp file".to_string(),
        )
            .into_response();
    }

    // Update stats
    {
        let mut map = state.stats.lock().unwrap();
        *map.entry(payload.language.to_lowercase()).or_insert(0) += 1;
        drop(map);
    }

    let mut cmd = Command::new("sudo");
    cmd.args([
        "-u",
        "code_runner",
        script_path,
        code_path.to_str().unwrap(),
    ]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = match timeout(Duration::from_secs(15), cmd.output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Execution failed: {}", e),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::REQUEST_TIMEOUT,
                "Execution timed out".to_string(),
            )
                .into_response();
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status_code = output.status.code().unwrap_or(-1);

    let json = Json(ExecuteResponse {
        stdout,
        stderr,
        status: status_code,
    });

    (StatusCode::OK, json).into_response()
}

#[tokio::main]
async fn main() {
    let stats: Stats = Arc::new(Mutex::new(HashMap::new()));
    let state = AppState { stats };

    let app = Router::new()
        .route("/execute", post(execute_code))
        .route("/stats", get(stats_handler))
        .with_state(state);

    let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 3000);
    println!("Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
