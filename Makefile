
PROTOC ?= protoc

generate-messages:
	$(PROTOC) --rust_out src/message/ src/message/*.proto
