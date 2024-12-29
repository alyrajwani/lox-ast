VERSION=1.0
NAME=lox-ast
EXEC=rust-exec
PREFIX=$(HOME)/.local

all: check clean build run

default: help

build: 
	@echo "> Building expr.rs..."
	@cargo build
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
	@echo "> Usage: \"make <build> <clean> <check> <compile> <run> <all>\""

