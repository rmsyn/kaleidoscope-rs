# Simple makefile to build the firmware

all:
	@cargo build -Zbuild-std=core --release

Phony: all
