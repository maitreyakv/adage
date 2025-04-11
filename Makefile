CARGO_CHECK_CMD = check --all-targets --workspace
CARGO_TEST_CMD = test --workspace --all-targets
CARGO_CLIPPY_CMD = clippy --all-targets --workspace -- -D warnings
CARGO_FMT_CMD = fmt --check --all

clean:
	cargo clean

lint:
	cargo $(CARGO_FMT_CMD)

check:
	cargo $(CARGO_CHECK_CMD)
	cargo $(CARGO_CLIPPY_CMD) 

test:
	cargo $(CARGO_TEST_CMD)

watch:
	cargo watch --no-restart --clear \
		-x "$(CARGO_FMT_CMD)" \
		-x "$(CARGO_CLIPPY_CMD)" \
		-x "$(CARGO_CHECK_CMD)" \
		-x "$(CARGO_TEST_CMD)"
