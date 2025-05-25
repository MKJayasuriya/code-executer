#!/bin/bash
set -e

INPUT_CODE_FILE="$1"
SANDBOX_DIR="/home/code_runner/code_runs"

# Extract the public class name from the code
CLASS_NAME=$(grep -oP 'public\s+class\s+\K\w+' "$INPUT_CODE_FILE" | head -n1)

if [ -z "$CLASS_NAME" ]; then
  echo "Error: No public class found." >&2
  exit 1
fi

CODE_FILE="$SANDBOX_DIR/${CLASS_NAME}.java"
cp "$INPUT_CODE_FILE" "$CODE_FILE"

# Compile
javac "$CODE_FILE"

# Run
timeout 5s java \
  -XX:CompressedClassSpaceSize=128m \
  -XX:MaxMetaspaceSize=128m \
  -Xmx256m \
  -cp "$SANDBOX_DIR" "$CLASS_NAME"

# Cleanup
rm -f "$CODE_FILE" "${CODE_FILE%.java}.class"
