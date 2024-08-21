#!/bin/bash
for program in programs/*; do
    cargo build-sbf --manifest-path "$program/Cargo.toml"
done
