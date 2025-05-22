# Code Execution Service

This project provides a REST API for executing code snippets in multiple programming languages (Python, C++, JavaScript, Java) inside Docker containers for isolation and security.

## Features

- Accepts code and language via HTTP POST.
- Runs code in a resource-limited Docker container.
- Supports Python, C++, JavaScript, and Java.
- Returns both stdout and stderr from the code execution.
- Cleans up temporary files after execution.

## How It Works

1. **API Endpoint**:  
   - `POST /execute`  
   - Accepts a JSON payload:
     ```json
     {
       "language": "python|cpp|javascript|java",
       "code": "<code string>"
     }
     ```

2. **Processing Flow**:
   - The server receives the code and language.
   - It writes the code to a temporary file with a unique name and appropriate extension.
   - It selects a Docker image based on the language:
     - `code-runner-python` for Python
     - `code-runner-cpp` for C++
     - `code-runner-js` for JavaScript
     - `code-runner-java` for Java
   - The code file is mounted into the Docker container at `/code/user.<ext>`.
   - The container is run with strict resource limits (memory, CPU, no network).
   - The output (stdout and stderr) is captured and returned in the API response.
   - The temporary file is deleted after execution.

3. **Security**:
   - Code runs in a Docker container with:
     - No network access
     - Limited memory (100MB)
     - Limited CPU (0.5 cores)
     - Limited number of processes (50)
   - Temporary files are cleaned up after execution.

## Example Request

```bash
curl -X POST http://localhost:3000/execute \
  -H "Content-Type: application/json" \
  -d '{"language":"python","code":"print(2+2)"}'
```

## Example Response

```
Output:
4

Error:
```

## Project Structure

- `src/main.rs` — Main server code (Axum + Tokio).
- Docker images (must be available locally):
  - `code-runner-python`
  - `code-runner-cpp`
  - `code-runner-js`
  - `code-runner-java`

## Requirements

- Rust (for building the server)
- Docker (for running code securely)
- The required Docker images must be built or pulled in advance.

## Building Docker Images

Before running the server, you must build the Docker images for each supported language.  
A helper Rust script is provided for this purpose:

```bash
rustc build_images.rs 
./build_images  
```

This will build the following images using their respective Dockerfiles:
- `code-runner-python` (from `Dockerfiles/python.Dockerfile`)
- `code-runner-cpp` (from `Dockerfiles/cpp.Dockerfile`)
- `code-runner-js` (from `Dockerfiles/js.Dockerfile`)
- `code-runner-java` (from `Dockerfiles/java.Dockerfile`)

Make sure the `Dockerfiles/` directory exists and contains the necessary Dockerfiles.

## Running the Server

```bash
cargo run
```

The server will listen on `0.0.0.0:3000`.

## Extending

To add support for more languages:
- Add a new entry in the language/image/ext mapping in `main.rs`.
- Provide a corresponding Docker image.

---

## GitHub Description

A secure, containerized code execution REST API supporting Python, C++, JavaScript, and Java.  
- Accepts code and language via HTTP POST.
- Runs code in resource-limited Docker containers for isolation.
- Returns both stdout and stderr.
- Easily extensible for more languages.
- Includes helper script to build all required Docker images.

Ideal for online judges, code playgrounds, or educational tools.

## License

MIT License.
