
PROTOC ?= protoc

generate-messages:
	$(PROTOC) --proto_path src/message --rust_out src/message/ src/message/*.proto
