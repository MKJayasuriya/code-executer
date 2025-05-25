#!/bin/bash
ulimit -t 5
ulimit -v 1000000
ulimit -f 10000
node "$1"
