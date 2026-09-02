#!/bin/bash

cargo build --release
gcc examples/c_example.c -I include -L target/release -lrust_avl -o example
LD_LIBRARY_PATH=target/release ./example
rm example