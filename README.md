# Rust HLS streamer



## Prerequisites

Ensure you have the following installed:
- [Docker](https://www.docker.com/get-started)
- [Docker Compose](https://docs.docker.com/compose/install/)
- [rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## How to Run

```bash
git clone https://github.com/glebchanskiy/hls-experimental-server
cd hls-experimental-server
```

Run MinIO:
```bash
docker compose up -d
```

Run server:
```bash
cargo run
```

And open the file `player.html` in browser.
