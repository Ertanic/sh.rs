# About

**sh.rs** is an open-source link shortener toy project. Due to the fact that this is a toy project, it will not develop in any way.

The focus was on making it easier to redesign a project without having to build it. In other words, all HTML templates and styles are included with the server.

# Build

In order to build the server, you will need the Rust [toolchain](https://www.rust-lang.org/). Once you have installed the toolchain, you can start building the server:

```powershell
cargo build --release
```

# Run

## Manually

Starting the server is more complicated, as it requires [PostgreSQL](https://www.postgresql.org/download/) and [Redis](https://redis.io/downloads/). Once you have set them up, you need to pass the connection strings to the server. This can be done either through the environment variables `DATABASE_URL` and `REDIS_URL`, or through the config file:

```toml
[server]
redis = "redis://localhost:6379"
database = "postgres://user:password@localhost:5432/sh_rs"
```

## Docker

The project already contains some Docker files, but they are intended solely for testing purposes and require further refinement for use in production.

For information on how to launch and work with Docker containers, please refer to the [documentation](https://docs.docker.com/reference/cli/docker/compose/up/).

# Licensing

Absolutely no license, absolute freedom, you can use this code for any purpose.