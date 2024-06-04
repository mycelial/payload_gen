DB_USER ?= "root"
DB_PASSWORD = "root"

.PHONY: dev
dev:
	cargo-watch -q -c -s 'cargo run'

.PHONY: run_postgres
run_postgres:
	docker run --name postgres --rm -e POSTGRES_USER=$(DB_USER) -e POSTGRES_PASSWORD=$(DB_PASSWORD) -p 5432:5432 -d postgres


.PHONY: attach_postgres
attach_postgres:
	docker exec -it -e PGPASSWORD=$(DB_PASSWORD) postgres psql -h 127.0.0.1 -U $(DB_USER)
 
.PHONY: stop_postgres
stop_postgres:
	docker rm -f postgres
