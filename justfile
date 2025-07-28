# Justfile for managing build tasks
# Use `just <task>` to run a specific task
# Use `just --summary` to see available tasks
# QEMU
qemu *args: build
  ./scripts/run-qemu.sh {{exe}} {{args}}
qemu-debug *args: build
  ./scripts/run-qemu.sh --debug {{exe}} {{args}}

disassemble: build
  scripts/disassemble.sh target/armv7-unknown-linux-gnueabihf/debug/examples/armv7

exe := "target/armv7-unknown-linux-gnueabihf/debug/examples/armv7"

build: debug

run:
  cargo run

release:
  cargo build --release

debug:
  cargo build --example armv7

test:
  cargo test

clean:
  cargo clean

