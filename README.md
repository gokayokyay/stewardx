# StewardX

**S**cheduled **T**ask **E**xecutor **W**ith **A**synchronous **R**untime and **D**atabase **X**

<sub><sup>*Because all the cool stuff have an X...*</sup></sub>

StewardX is a task scheduler written in pure [Rust](https://www.rust-lang.org/). By leveraging [tokio](https://tokio.rs/), it is asynchronous and blazing-fast!

*This project is under heavy development, it can (and probably will) break. Please do not use in production yet*

### Features
 - It's pure Rust and *lightweight*.
 - Uses PostgreSQL as database to persist the tasks.
 - Has multiple task types, currently it supports `command` and `docker` tasks.
 - By leveraging *traits*, it's a joy to extend StewardX.
 - Supports both **Dockerfiles** and pre-built **Docker images**.
 - Has multiple task frequencies, currently it supports `cron` and `hook`.
 - Stores outputs of tasks.

### Roadmap to v1
- [ ] Build a minimal control panel
- [ ] Cover errors
- [ ] Test coverage
- [ ] Write documentation
- [X] Use features to make panel and docker features optional

### Roadmap not in order
- [ ] Add websocket support
- [ ] Add one or more task types
- [ ] Create a CLI app
- [ ] Create a cute logo
- [ ] Create examples
- [ ] Create a landing page
