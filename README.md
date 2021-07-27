[![][Logo]][Website] 
# StewardX

**S**cheduled **T**ask **E**xecutor **W**ith **A**synchronous **R**untime and **D**atabase **X**

<sub><sup>*Because all the cool stuff have an X...*</sup></sub>

StewardX is a task scheduler written in pure [Rust](https://www.rust-lang.org/). By leveraging [tokio](https://tokio.rs/), it is asynchronous and blazing-fast!

*This project is under heavy development, it can (and probably will) break. Please do not use in production yet*

### Table of Contents
  + [Related repositories](#related-repositories)
  + [Features](#features)
  + [Tutorials](#tutorials)
  + [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
  + [Roadmap to v1](#roadmap-to-v1)
  + [Roadmap not in order](#roadmap-not-in-order)
  + [License](#license)

### Related repositories
| Repository                                                   | Description                    |
|--------------------------------------------------------------|--------------------------------|
| [Panel](https://github.com/gokayokyay/stewardx-panel)        | Proof of concept control panel |
| [Documentation](https://github.com/gokayokyay/stewardx-docs) | Documentation repository       |
| [CLI](https://github.com/gokayokyay/stewardx-cli)            | StewardX CLI                   |

### Features
 - It's pure Rust and *lightweight*.
 - Uses PostgreSQL as database to persist the tasks.
 - Has multiple task types, currently it supports `command` and `docker` tasks.
 - By leveraging *traits*, it's a joy to extend StewardX.
 - Supports both **Dockerfiles** and pre-built **Docker images**.
 - Has multiple task frequencies, currently it supports `cron` and `hook`.
 - Stores outputs of tasks.

### Tutorials

- Building your own CI [here](https://stewardx.dev/Tutorials/tutorial-ci) or [here](https://dev.to/gokayokyay/build-your-own-react-ci-in-5-minutes-1aen)

### Getting Started
#### Prerequisites
- Docker (optional)
- Rust
- git

Until the prebuilt binaries are released, (Linux x64 binaries are [released](https://github.com/gokayokyay/stewardx/releases/latest) WOOT 🥳) you can build StewardX on your own.
First install the Rust itself by following the instructions on: [Rustup](https://rustup.rs/)
Then clone the repository
```bash
git clone https://github.com/gokayokyay/stewardx
cd stewardx
```

If you don't want to use Docker, then please disable the `docker` feature in Cargo.toml in the root of repository. To disable it, just remove the "docker" item from the `default` key of `[features]`. So it'll look like
```toml
default = ["panel", "cmd"]
```

You'll need a running Postgres instance. If you got one, you can skip this step. But if you don't, there're some utility scripts in scripts folder located in the root of the repository. For simplicity's sake, let's just use the temporary one, `docker-postgres-temp.sh`.
```bash
chmod +x ./scripts/docker-postgres-temp.sh
./scripts/docker-postgres-temp.sh
```
When your instance is up and running, you'll need to state some environment variables.
- STEWARDX_DATABASE_URL
- DATABASE_URL - this one won't be required when prebuilt binaries are released. It's required for now because of [SQLx](https://github.com/launchbadge/sqlx).

Let's define those variables:
```bash
# Replace the db url with your own if you skipped previos step
export DATABASE_URL=postgresql://postgres:1234@localhost:5432/postgres
export STEWARDX_DATABASE_URL=postgresql://postgres:1234@localhost:5432/postgres
```

Awesome! Now we just run:
```bash
# Note: This may take a while!
cargo build --release
```

If you get this error:
```bash
error: linker `cc` not found
```

Then install this package (Debian/Ubuntu), then issue the previous command.
```bash
sudo apt install build-essential
```

And while you're at it, you may want to install these packages too:
```bash
sudo apt install libssl-dev pkg-config
```

When to compilation is finished, now you can start StewardX with:
```bash
./target/release/stewardx
```

Now add your first `CmdTask` with frequency of `Hook` (Basically a webhook). From another terminal run:
```bash
curl --header "Content-Type: application/json" -X POST --data '{"task_name": "My test task", "frequency": "Hook", "task_type": "CmdTask", "task_props": {"command":"echo Hello StewardX!"}}' http://localhost:3000/tasks
```

This command will output an id, save it somewhere, mine was "08234e0c-63b8-420a-a4fc-80691ca86e17". To execute your previous task, run:
```bash
# curl --header "Content-Type: application/json" -X POST http://localhost:3000/#id from previous step
curl --header "Content-Type: application/json" -X POST http://localhost:3000/execute/08234e0c-63b8-420a-a4fc-80691ca86e17
```

You should get this response:
```json
{"status":"success"}
```

Awesome! You may be wondering, where's the output of the task? It's easy! In the database. Just run:
```bash
curl --header "Content-Type: application/json" http://localhost:3000/task/#your task id#/reports
```

And you'll get your execution report :)

### Roadmap to v1
- [X] Build a minimal control panel
- [X] Cover errors
- [ ] Test coverage
- [X] Write documentation - [StewardX Docs](https://github.com/gokayokyay/stewardx-docs)
- [X] Use features to make panel and docker features optional
- [X] Precompiled binaries (Currently linux only)
- [ ] Create distributed worker system

### Roadmap not in order
- [ ] Add websocket support
- [ ] Add one or more task types
- [X] Create a CLI app - 🥳
- [X] Create a cute logo - 🥳
- [ ] Create examples
- [ ] Create a landing page
- [ ] Create a GUI controller
- [ ] Ability to delay tasks' execution?



## License

Licensed under

-   Apache License, Version 2.0
    ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)


[Website]: https://stewardx.dev
[Logo]: https://stewardx.dev/img/stewardx-logo.svg
