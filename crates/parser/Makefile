.PHONY: help
help:  ## Show help messages for make targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'
check: ## check
	cargo check
run: check ## run
	cargo run
test: check## test
	cargo test -- --nocapture
doc: ## doc
	# cargo doc --all-features --no-deps --open
	cargo doc --all-features --open
update: ## update
	cargo update
