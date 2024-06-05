SHELL := bash

DB_USER ?= "root"
DB_PASSWORD ?= "root"
ORIGIN ?= "metrics"
BATCH_SIZE ?= 4096

OPENSSL_VERSION = 3.3.1
OPENSSL_URL = "https://www.openssl.org/source/openssl-$(OPENSSL_VERSION).tar.gz"
OPENSSL_ARCHIVE = $(shell basename $(OPENSSL_URL))
OPENSSL_FOLDER = $(shell basename $(OPENSSL_ARCHIVE) .tar.gz)
OPENSSL_PREFIX = $(PWD)/openssl
TARGET ?= x86_64-unknown-linux-gnu

.PHONY: dev
dev:
	cargo-watch -q -c -s "cargo run -r -- --origin=$(ORIGIN) --batch-size=$(BATCH_SIZE)"

.PHONY: run_postgres
run_postgres:
	docker run --name postgres --rm -e POSTGRES_USER=$(DB_USER) -e POSTGRES_PASSWORD=$(DB_PASSWORD) -p 5432:5432 -d postgres


.PHONY: attach_postgres
attach_postgres:
	docker exec -it -e PGPASSWORD=$(DB_PASSWORD) postgres psql -h 127.0.0.1 -U $(DB_USER) -d postgres
 
.PHONY: stop_postgres
stop_postgres:
	docker rm -f postgres


.PHONY: sh_postgres
sh_postgres:
	docker exec -it -u root postgres bash

.PHONY: build_x64
build_x64: export TARGET=x86_64-unknown-linux-gnu
build_x64: export OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/$(OPENSSL_VERSION)/$(TARGET)/include
build_x64: export OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)/$(OPENSSL_VERSION)/$(TARGET)/lib
build_x64: export OPENSSL_STATIC=1
build_x64: export CC_$(TARGET)=$(TARGET)-cc
build_x64: export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=$(TARGET)-gcc
build_x64: 
	cargo build -r --target $(TARGET)


.PHONY: cross_openssl
cross_openssl: export CC="$(TARGET)-gcc"
cross_openssl: export CXX="$(TARGET)-g++"
cross_openssl: export AS="$(TARGET)-as"
cross_openssl: export AR="$(TARGET)-ar"
cross_openssl: export NM="$(TARGET)-nm"
cross_openssl: export RANLIB="$(TARGET)-ranlib"
cross_openssl: export LD="$(TARGET)-ld"
cross_openssl: export STRIP="$(TARGET)-strip"
cross_openssl:
	mkdir -p $(OPENSSL_PREFIX)
	wget $(OPENSSL_URL) -O $(OPENSSL_PREFIX)$(OPENSSL_ARCHIVE)
	tar -xzf $(OPENSSL_PREFIX)$(OPENSSL_ARCHIVE) -C openssl
	cd openssl/$(OPENSSL_FOLDER) \
	&& ./Configure linux-generic64 shared \
		--prefix=$(OPENSSL_PREFIX)/$(OPENSSL_VERSION)/$(TARGET)  \
		--openssldir=$(OPENSSL_PREFIX)/$(OPENSSL_VERSION)/$(TARGET) \
	&& make -j 8  \
	&& make install
