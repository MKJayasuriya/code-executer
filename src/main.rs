use axum::{extract::Json, routing::post, Router};
use serde::Deserialize;
use std::fs;
use std::env;
use uuid::Uuid;
use tokio::process::Command as TokioCommand;

#[derive(Deserialize)]
struct CodePayload {
    language: String,
    code: String,
}

async fn execute_code(Json(payload): Json<CodePayload>) -> String {
    println!("Received payload: language={}, code_len={}", payload.language, payload.code.len());
    let tmp_dir = env::temp_dir();
    let uid = Uuid::new_v4().to_string();
    let (ext, image) = match payload.language.as_str() {
        "python" => ("py", "code-runner-python"),
        "cpp" => ("cpp", "code-runner-cpp"),
        "javascript" => ("js", "code-runner-js"),
        "java" => ("java", "code-runner-java"),
        _ => return "Unsupported language".into(),
    };
    let code_file = tmp_dir.join(format!("user_{}.{}", uid, ext));
    println!("Writing code to: {:?}", code_file);
    if let Err(e) = fs::write(&code_file, &payload.code) {
        println!("Failed to write code file: {}", e);
        return format!("Failed to write code file: {}", e);
    }

    let value = format!("{}:/code/user.{}", code_file.to_string_lossy(), ext);
    println!("Docker volume mapping: {}", value);
    println!("Using docker image: {}", image);
    let docker_args = vec![
        "run",
        "--rm",
        "--network", "none",
        "--memory=100m",
        "--cpus=0.5",
        "--pids-limit=50",
        "-v",
        &value,
        image,
    ];

    println!("Running docker with args: {:?}", docker_args);

    let output = TokioCommand::new("docker")
        .args(&docker_args)
        .output()
        .await;

    // Clean up temp file
    let _ = fs::remove_file(&code_file);

    match output {
        Ok(out) => {
            println!("Docker stdout: {}", String::from_utf8_lossy(&out.stdout));
            println!("Docker stderr: {}", String::from_utf8_lossy(&out.stderr));
            format!(
                "Output:\n{}\nError:\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        },
        Err(e) => {
            println!("Failed to run docker: {}", e);
            format!("Failed to run docker: {}", e)
        },
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/execute", post(execute_code));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
