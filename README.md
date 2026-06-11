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

## Touch Control

The device screen uses vendored `interact.js` 1.10.27 for unified mouse/touch
tap and drag gesture handling. Android commands still use `adb` underneath; the
deeper low-latency Android integration target is `scrcpy` or
`@yume-chan/scrcpy`.
