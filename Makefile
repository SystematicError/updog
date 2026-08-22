EXE = updog
TARGET := $(shell rustc --print=host-tuple)

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
	MOVE := move /Y
else
	NAME := $(EXE)
	MOVE := mv
endif


.PHONY: openbench
openbench:
	cargo rustc --release -- -C target-cpu=native --emit link="$(NAME)"

.PHONY: pgo
pgo:
	cargo pgo run -- bench
	cargo pgo optimize
	$(MOVE) "target/$(TARGET)/release/$(EXE)" "$(NAME)"

.PHONY: native
native:
	RUSTFLAGS="-C target-cpu=native" cargo pgo build
	$(MAKE) pgo

.PHONY: bench
bench:
	cargo rustc --release -- -C target-cpu=native
	"target/release/$(NAME)" bench
