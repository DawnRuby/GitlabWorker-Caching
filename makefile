prog :=xnixperms
debug ?=

ifdef debug
  $(info debug is set to $(debug), running debug version)
  release :=
  target :=debug
  extension :=debug
else
  $(info debug is set to $(debug), running release version)
  release :=--release
  target :=release
  extension :=
endif

build:
	cargo build $(release)

clean:
	cargo clean $(release)
 
help:
	@echo "usage: make $(prog) [debug=1]"