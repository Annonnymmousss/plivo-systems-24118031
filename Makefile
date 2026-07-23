CARGO ?= cargo

.PHONY: all clean

all:
	$(CARGO) build --release --bins
	cp target/release/sender ./sender
	cp target/release/receiver ./receiver

clean:
	$(CARGO) clean
	rm -f sender receiver
