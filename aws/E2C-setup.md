# AWS EC2 Setup Guide for Code Executer

This guide helps you set up an Ubuntu 22.04+ EC2 instance to securely run code in multiple languages.

---

## 1. SSH into your EC2 Instance

```sh
ssh -i your-key.pem ubuntu@your-ec2-ip
```

---

## 2. System Update & Essential Packages

```sh
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential openjdk-17-jdk g++ nodejs npm python3 python3-pip firejail git curl nginx ufw
```

---

## 3. Create Restricted User for Code Execution

```sh
sudo useradd -m code_runner
sudo passwd -l code_runner  # lock password, no login
```

---

## 4. Prepare Directories

```sh
sudo mkdir -p /code_runner/code_runs
sudo chown -R code_runner:code_runner /code_runner/code_runs
```

---

## 5. Set Up Runners Directory

```sh
sudo mkdir -p /runners
# Copy your runner scripts (e.g., java_runner.sh, python_runner.sh, etc.) to /runners
sudo cp -rf /home/ubuntu/runners/*.sh /runners/
sudo chown -R code_runner:code_runner /runners
sudo chmod -R 755 /runners
sudo chmod 700 /runners/*.sh
```

---

## 6. Configure Sudoers for Secure Script Execution

```sh
echo 'code_runner ALL=(ALL) NOPASSWD: /runners/*' | sudo tee /etc/sudoers.d/code_runner
sudo chmod 440 /etc/sudoers.d/code_runner
```

---

## 7. Enable and Configure Firewall

```sh
sudo ufw allow OpenSSH
sudo ufw allow 8080
sudo ufw enable
```

---

## 8. Running the Server

- Upload your compiled server binary (e.g., `execute-test-v2`) to the instance.
- Make it executable:

```sh
chmod +x execute-test-v2
```

- Run the server (use `nohup` or `tmux` for background):

```sh
./execute-test-v2
```

---

## 9. Permissions and Ownership

If you update runner scripts or binaries, ensure correct permissions:

```sh
sudo chown code_runner:code_runner /runners/*.sh
sudo chmod 700 /runners/*.sh
```

---

## 10. Useful Commands

- List runner scripts: `ls /runners/`
- Check code_runs dir: `ls -l /code_runner/code_runs`
- Switch to code_runner: `sudo su - code_runner`
- Kill server: `killall -9 execute-test-v2`

---

**Note:**  
- All runner scripts must be in `/runners` and owned by `code_runner`.
- The server expects to find runner scripts in `/runners/` (update your app config if needed).
- Adjust paths if your deployment differs.

---

**EC2 setup complete. Your server should now be ready to securely execute code!**

