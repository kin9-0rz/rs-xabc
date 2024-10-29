build:
	cargo b
clean:
	cargo clean
test: build
	@./target/debug/xabc -h
install: clean
	cargo install --path xabc
doc: clean
	cargo doc --workspace --exclude xabc --no-deps --open 
