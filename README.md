# StewardX

**S**cheduled **T**ask **E**xecutor **W**ith **A**synchronous **R**untime and **D**atabase **X**

<sub><sup>*Because all the cool stuff have an X...*</sup></sub>

StewardX is a task scheduler written in pure [Rust](https://www.rust-lang.org/). By leveraging [tokio](https://tokio.rs/), it is asynchronous and blazing-fast!

### Features
 - It's pure Rust and *lightweight*.
 - Uses PostgreSQL as database to persist the tasks.
 - Has multiple task types, currently it supports `command` and `docker` tasks.
 - By leveraging *traits*, it's a joy to extend StewardX.
 - Supports both **Dockerfiles** and pre-built **Docker images**.
 - Has multiple task frequencies, currently it supports `cron` and `hook`.
 - Stores outputs of tasks.


