# proxy-caller

### Create or drop the schema
```bash  
docker compose exec -it postgres bash

psql -U postgres

CREATE DATABASE caller ENCODING = 'UTF8';

sea-orm-cli migrate up -u postgres://postgres:omnity_go@localhost/caller

sea-orm-cli generate entity -u postgres://postgres:omnity_go@localhost:5432/caller -o src/entities
```

### Build and run

```bash
cargo build --locked --release -p proxy-caller

# start sync
./target/release/proxy_caller
```

### Config identity
```bash  
export DFX_IDENTITY=$(<./test.pem)
export DATABASE_URL=postgres://postgres:omnity_go@localhost/caller
export DFX_NETWORK=https://ic0.app
export CKBTC_CANISTER_ID=mxzaz-hqaaa-aaaar-qaada-cai
```