#!/bin/bash

if [[ -z "$1" ]]; then
    cargo run --release
elif [[ "profile" ]]; then
    cargo run --release --features bevy/trace_chrome
fi