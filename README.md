# axum-kit

Streamline the integration and usage of [Axum](https://github.com/tokio-rs/axum) with [SQLx](https://github.com/launchbadge/sqlx) and [Redis](https://github.com/redis-rs/redis-rs).

Without further ado, please see the [demo](examples/demo.rs).

## Usage

To use `axum-kit`, add the following to your `Cargo.toml`:

```toml
[dependencies]
axum-kit = { git = "https://github.com/4lkaid/axum-kit.git" }
```

## Example Configuration File

```toml
[general]
listen = "0.0.0.0:8000"

[logger]
# Log levels: trace > debug > info > warn > error
# trace: Very detailed debugging information.
# debug: General debugging information.
# info: Normal operational information.
# warn: Potential issues.
# error: Serious problems.
level = "debug"
# writer options:
# file: Logs to "directory/file_name_prefix.year-month-day".
# stdout: Logs to console.
writer = "file"
directory = "./log"
file_name_prefix = "axum_kit.log"

[postgres]
url = "postgres://postgres:@127.0.0.1:5432/postgres"
max_connections = 10
min_connections = 1
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 1800

[redis]
url = "redis://127.0.0.1:6379"

```

## License

This project is licensed under the [MIT license](LICENSE).
