# mkwpp-api-rust
Faster non-Minimum Viable Product API

## Setup
First off, log into the postgres user. You want to create a database (to install postgres, visit [this link](https://www.postgresql.org/download/)):
```bash
createdb database_name
```
You can then log out of the postgres user.

Build the executable with build.sh.

Create a `.env` file. (The possible parameters for it are below).

With `cargo` (to install it, visit [this link](https://rustup.rs/)) you want to download `sqlx-api`:
```bash
cargo install sqlx-cli
```

To run migrations, you'll have to type
```bash
sqlx database setup --source db/migrations
```

To import Fixtures instead, you should run the executable with the arguments `tables old` for Fixtures created by the old Django Database

```bash
./mkwpp-api-rust import old
```

## Extra info
Not everything is actually stored in PSQL like the Python+Django backend, as that's not actually needed (how often will we actually change the `cups` table, lets be real).

This will be actually faster, by the nature of not being an interpreted language.

## Possible .env Parameters
| Key | Value Type | Description | Default |
|-----|------------|-------------|---------|
| USERNAME | String | Database admin username | postgres |
| PASSWORD | String | Database admin password | password |
| DATABASE_NAME | String | Database name | mkwppdb |
| HOST | String | Database IP Hostname | localhost |
| PORT | String | Database IP Port | 5432 |
| DATABASE_URL | String | URL to the Database, can also be generated with the above keys | postgres://postgres:password@localhost:5432/mkwppdb |
| MAX_CONN | u32 | Connections in the Connection Pool | 25 |
| KEEP_ALIVE | u64 | Time for which a URL should hot reload, in milliseconds | 60000 |
| CLIENT_REQUEST_TIMEOUT | u64 | Max time a request should take before being dropped | 120000 |