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
./target/release/proxy-caller

```