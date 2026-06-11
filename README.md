# Appmo

Rust web application for controlling Android emulators and iOS simulators running on the same Mac.

## Run

```sh
cp .env.example .env
cargo run -p appmo-server
```

Open:

```text
http://<mac-lan-ip>:8080
```

The web UI and API are intentionally unauthenticated for local/LAN development.
