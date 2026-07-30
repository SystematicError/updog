EXE = updog

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

.PHONY: openbench
openbench:
	cargo rustc --release -- -C target-cpu=native --emit link="$(NAME)"

.PHONY: bench
bench:
	cargo rustc --release -- -C target-cpu=native
	"target/release/$(NAME)" bench
