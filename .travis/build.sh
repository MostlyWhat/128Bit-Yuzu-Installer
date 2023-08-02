#!/usr/bin/env bash
cd /liftinstall || exit 1

yarn --cwd ui

cargo build --release
