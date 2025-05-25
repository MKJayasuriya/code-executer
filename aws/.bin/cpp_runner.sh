  GNU nano 7.2                                                              /opt/runners/cpp_runner.sh                                                                       
#!/bin/bash
set -e

INPUT_CODE_FILE="$1"

# Make sure file has .cpp extension (important for g++)
CODE_FILE=$(mktemp /home/code_runner/code_runs/code.XXXX.cpp)
EXEC_FILE=$(mktemp /home/code_runner/code_runs/a.out.XXXX)

# Copy source to temp file with proper extension
cp "$INPUT_CODE_FILE" "$CODE_FILE"
chmod 644 "$CODE_FILE"

echo "Compiling $CODE_FILE"

# 🔧 Compile and run
g++ -o "$EXEC_FILE" "$CODE_FILE"
"$EXEC_FILE"

# 🧹 Clean up
rm -f "$CODE_FILE" "$EXEC_FILE"
