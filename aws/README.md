# Code Executor API

A secure, concurrent code execution API supporting Python, JavaScript, Java, and C++.

## Features

- Execute code in multiple languages via HTTP API
- Per-language execution statistics (`/stats`)
- Secure execution using a restricted user (`code_runner`)
- Handles concurrency and timeouts

## Requirements

- Rust (for building the backend)
- Node.js (for load testing with k6)
- Docker (optional, for isolation)
- `sudo` and a non-privileged user named `code_runner`
- Runner scripts for each language in `/opt/runners/` (see below)
- Python, Node.js, Java, and g++ installed

## Setup

### 1. Create the `code_runner` User

```sh
sudo useradd -m -s /bin/bash code_runner
```

### 2. Place Runner Scripts

Create the following scripts in `/opt/runners/` and make them executable:

#### `/opt/runners/python_runner.sh`
```sh
#!/bin/bash
python3 "$1"
```

#### `/opt/runners/js_runner.sh`
```sh
#!/bin/bash
node "$1"
```

#### `/opt/runners/java_runner.sh`
```sh
#!/bin/bash
cd "$(dirname "$1")"
javac "$(basename "$1")" && java Main
```

#### `/opt/runners/cpp_runner.sh`
```sh
#!/bin/bash
src="$1"
bin="${src%.*}"
g++ "$src" -o "$bin" && "$bin"
```

```sh
sudo chmod +x /opt/runners/*.sh
sudo chown root:root /opt/runners/*.sh
```

### 3. Allow `code_runner` to Run Scripts via `sudo` Without Password

Edit `/etc/sudoers` (with `visudo`) and add:

```
# Allow running runner scripts as code_runner without password
youruser ALL=(code_runner) NOPASSWD: /opt/runners/python_runner.sh, /opt/runners/js_runner.sh, /opt/runners/java_runner.sh, /opt/runners/cpp_runner.sh
```

Replace `youruser` with the user running the Rust API.

### 4. Build and Run the Rust API

```sh
cd /home/jayasuriya/workspace/code-executer/aws
cargo build --release
./target/release/aws
```

The server will start on `http://0.0.0.0:3000`.

### 5. Test the API

You can use the provided `load-test.js` with [k6](https://k6.io/) for load testing:

```sh
npm install -g k6
snap install k6
k6 run /home/user/workspace/code-executer/aws/load-test.js
```

### 6. API Endpoints

- `POST /execute`  
  Request body:  
  ```json
  { "language": "python", "code": "print(123)" }
  ```
  Response: Output or error.

- `GET /stats`  
  Returns per-language execution counts.

## UI

A simple web UI is provided for testing the code execution API, built with Rust server-side rendering (SSR) using [Axum](https://github.com/tokio-rs/axum) and [askama](https://github.com/djc/askama).

Features:
- Select language (Python, JavaScript, Java, C++)
- Code editor (textarea)
- Output viewer
- Execute button calls the `/execute` API and displays the result

### Running the UI

1. Make sure the backend server is running.
2. Start the server (the UI is served at `/ui`):

    ```
    ./execute-test-v2
    ```

3. Open [http://localhost:3000/ui](http://localhost:3000/ui) in your browser.

## Security Notes

- All code runs as the restricted `code_runner` user.
- Temporary files are world-readable for execution, but are deleted after use.
- You may want to further sandbox or containerize execution for untrusted code.

## Troubleshooting

- Ensure runner scripts are executable and have correct permissions.
- Ensure `sudo` rules are set up for passwordless execution.
- Check logs for permission or path errors.

### Load testing
```sh
k6 run load-test.js --env BASE_URL=http://0.0.0.0:3000
```
---

**Production ready!**
