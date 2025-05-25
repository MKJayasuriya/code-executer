#!/bin/bash
ulimit -t 5       # CPU time limit in seconds
ulimit -v 1000000 # Memory limit ~1GB
ulimit -f 10000   # File size limit
python3 "$1"
