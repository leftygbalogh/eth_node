# Getting Started

## Project

Rust Terminal Snake Game (Greenfield).

## Prerequisites

1. Rust toolchain with cargo in PATH.
2. Bash runtime for launcher execution (Git Bash, Linux Bash, or WSL Bash).

## Build and Test

1. cargo test -q
2. cargo build --release

## Run

From repository root:

1. ./run_snake.sh

The launcher builds release binary if missing and routes runtime by shell environment.

## Runtime Artifacts

- Crash log: snake.log
- Leaderboard store: leaderboard.csv

## Governance Records

- prompts.md contains full prompt history.
- memory.md contains stage-by-stage status and decisions.
