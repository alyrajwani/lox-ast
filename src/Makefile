VERSION=1.0
NAME=rust-makefile
EXEC=rust-exec
PREFIX=$(HOME)/.local

all: check clean run

default: help

clean:
	@echo "> Cleaning build directory..."
	@rm -rf target/*
	@cargo clean
check:
	@echo "> Checking $(NAME)"
	@cargo check
compile:
	@echo "> Compiling program..."
	@cargo build
run:
	@echo "> Running program..."
	@clear
	@cargo run
help:
	@echo "> Usage: \"make [clean] [check] [compile] [run] [all]\""

