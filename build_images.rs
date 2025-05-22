use std::process::Command;

fn main() {
    let images = [
        ("python", "Dockerfiles/python.Dockerfile"),
        ("cpp", "Dockerfiles/cpp.Dockerfile"),
        ("javascript", "Dockerfiles/js.Dockerfile"),
        ("java", "Dockerfiles/java.Dockerfile"),
    ];

    for (name, dockerfile) in images.iter() {
        println!("Building image for {}...", name);
        let status = Command::new("docker")
            .args(&[
                "build",
                "-f",
                dockerfile,
                "-t",
                &format!("code-runner-{}", name),
                ".",
            ])
            .status()
            .expect("failed to execute docker build");

        if status.success() {
            println!("Built code-runner-{} successfully.", name);
        } else {
            eprintln!("Failed to build code-runner-{}.", name);
        }
    }
}
