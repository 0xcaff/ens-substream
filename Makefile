.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream:
	substreams run -e api.streamingfast.io:443 substreams.yaml block_to_owner_mappings -s 15792059 -t +1

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/ethereum,sf/substreams,google"

