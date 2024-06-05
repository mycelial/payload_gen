BATCH_SIZE ?= 32768
NUM_CHUNKS ?= 1000
NUM_LOADERS ?= 1
PG_HOST ?=
PG_PASSWORD ?=

.PHONY: run
run:
	@payload_gen \
		--batch-size $(BATCH_SIZE) \
		--num-loaders $(NUM_LOADERS) \
		--num-chunks $(NUM_CHUNKS) \
		-c 'postgres://postgres:$(PG_PASSWORD)@$(PG_HOST):5432/postgres?sslmode=require'

.PHONY: attach_postgres
attach_postgres:
	@psql 'postgres://postgres:$(PG_PASSWORD)@$(PG_HOST):5432/postgres'


.PHONY: attach_redshift
attach_redshift:
	@psql 'postgres://user:password@host:5439/database'
	
