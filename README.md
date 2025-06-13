# mkwpp-api-rust
Faster non-Minimum Viable Product API

## Setup
First off, log into the postgres user. You want to create a database (to install postgres, visit [this link](https://www.postgresql.org/download/)):
```bash
createdb database_name
```
You can then log out of the postgres user.

Build the executable with run-scripts/build.sh.

Create a `.env` file. (The possible parameters for it are below).

With `cargo` (to install it, visit [this link](https://rustup.rs/)) you want to download `sqlx-api`:
```bash
cargo install sqlx-cli
```

To run migrations, you'll have to type (this will be run every time the package is compiled)
```bash
sqlx database setup --source db/migrations --database-url database_url
```

To import Fixtures instead, you should run the executable with the arguments `import old` for Fixtures created by the old Django Database

```bash
./mkwpp-api-rust import old
```

## Extra info
Not everything is actually stored in PSQL like the Python+Django backend, as that's not actually needed (how often will we actually change the `cups` table, lets be real).

This will be actually faster, by the nature of not being an interpreted language.

## Possible .env Parameters
| Key | Value Type | Description | Default |
|-|-|-|-|
| DB_USERNAME | String | Database admin username | postgres |
| DB_PASSWORD | String | Database admin password | password |
| DB_NAME | String | Database name | mkwppdb |
| DB_HOST | String | Database IP Hostname | localhost |
| DB_PORT | u16 | Database IP Port | 5432 |
| DATABASE_URL | String | URL to the Database, can also be generated with the above keys | postgres://postgres:password@localhost:5432/mkwppdb |
| DB_MAX_CONN | u32 | Connections in the Connection Pool | 25 |
| SRV_KEEP_ALIVE | u64 | Time for which a URL should hot reload, in milliseconds | 60000 |
| SRV_CLIENT_REQUEST_TIMEOUT | u64 | Max time a request should take before being dropped | 120000 |

## TODO
- Submission Pages API
    - Endpoint for User's Submissions
    - Endpoint to Delete User's Submission
    - Endpoint to Create Submission
    - Endpoint for User's Edit Submissions
    - Endpoint to Delete User's Edit Submission
    - Endpoint to Create Edit Submission
- Account Actions
    - Creating Profiles
    - Claiming Profiles
- Admin API / Admin UI
    - Users UI
    - Players UI
    - Submissions UI
    - Scores UI
    - Standards UI
    - Tracks UI
    - Regions UI
    - Awards UI
    - News Updates UI
    - Logs UI
- SMTP Handling
    - Password reset
- Rewrite of /api/raw/ to account for HIDING DATA THAT SHOULD NOT BE VISIBLE WITHOUT AUTH
- Rewrite of Standard Levels / Standards handling to avoid hardcoded values
- Various optimizations and removing duplicate code