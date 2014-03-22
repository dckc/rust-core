SHELL := /bin/bash

SOURCES = $(wildcard *.rs)
BINARIES = $(patsubst %.rs,%,$(SOURCES))
INTERMEDIATES = $(patsubst %.rs,%.bc,$(SOURCES))

all: $(BINARIES)

%: %.rs core
	rustc $< --emit=bc --cfg libc -O -Z no-landing-pads -Z lto -L .
	clang $@.bc -o $@ -O2 -lpthread
	./$@

core:
	rustc ../core/lib.rs --out-dir . # test that the freestanding subset builds
	rustc --cfg libc ../core/lib.rs --out-dir . -O -Z no-landing-pads
	touch core

clean:
	rm -f $(BINARIES) $(INTERMEDIATES) *.rlib core
