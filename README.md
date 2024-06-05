## Simple payload generator section


### How to run:
1. Setup postgres via `make run_postgres`, docker is required
2. Attach to database via `make attach_postgres` 
3. Create `metrics table`: `create table metrics(id bigserial, value text);`
4. Run load: `cargo run -- --batch-size 10000`

### How to build:
Assuming mac os:
1. run `make cross_openssl`
2. run `make build_x64`
3. scp final binary from `target/x86../release/payload_gen` to where it's needed
