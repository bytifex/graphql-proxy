#!/bin/sh

cd $(dirname $0)

cargo run --release -- serve -l localhost:8000
