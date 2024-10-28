build:
	cargo b
	./target/debug/xabc -h
test: build
	@./target/debug/xabc -p crates/parser/fixtures/demo.abc -c -m -s
install:
	cargo install --path xabc
