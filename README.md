# Appmo

Rust web application for controlling Android emulators and iOS simulators running on the same Mac.

## Run

```sh
cp .env.example .env
cargo run -p appmo-server
```

On startup Appmo checks the local control tools and auto-installs missing
developer dependencies when possible. It can install Android platform tools with
Homebrew, plus `idb-companion` and `fb-idb` for iOS full touch control, and
`ffmpeg` for WebRTC video preview. Disable this bootstrap with:

```sh
APPMO_AUTO_INSTALL_DEPS=false cargo run -p appmo-server
```

Open:

```text
http://<mac-lan-ip>:8080
```

The web UI and API are intentionally unauthenticated for local/LAN development.

UDP control listens on the next port after `APPMO_BIND` by default. For example,
`APPMO_BIND=127.0.0.1:18081` enables UDP control on `127.0.0.1:18082`.
Override it with `APPMO_UDP_BIND=host:port`, or disable it with
`APPMO_UDP_BIND=off`.

```sh
printf '{"request_id":"tap-1","device_id":"android:emulator-5554","type":"tap","x":320,"y":640}' \
  | nc -u -w1 127.0.0.1 18082
```

## Touch Control

The device screen uses vendored `interact.js` 1.10.27 for unified mouse/touch
tap and drag gesture handling. Control commands are sent over WebSocket when
available, with REST as a compatibility fallback. Android input keeps a
persistent `adb shell` session per device and writes `input ...` commands into
that session, avoiding a new `adb` process for every tap/swipe/key event.

iOS simulator preview uses `xcrun simctl io <udid> screenshot --type=jpeg -`,
which streams compact screenshot bytes through stdout instead of writing a
temporary file. Full iOS simulator touch control follows the same approach as
ios-bridge: Appmo uses `idb ui tap/swipe/text/key/button --udid <udid>` and maps
browser screenshot pixels into iOS point coordinates from `idb describe`.
At startup Appmo auto-installs `idb-companion` and `fb-idb` when `idb` is
missing. You can still set `APPMO_IDB_PATH` if `idb` lives outside `PATH`.
When `idb` is unavailable, tap falls back to Simulator-window AppleScript
control, but swipe/text/key need `idb` for full fidelity.

Screen preview defaults to the conservative one-shot screenshot path with
adaptive polling, adjustable FPS, preloaded image swaps, and no overlapping
fetches so control commands keep room to run. For lowest-latency interactive
preview, choose WebRTC mode: Appmo first negotiates a VP8 media track so the
browser can use its native video decoder, with `ffmpeg` running as a persistent
realtime encoder. If media-track negotiation or encoding is unavailable, Appmo
falls back to the WebRTC `appmo-preview` data channel, then to the Rust-served
multipart stream. `format=auto` keeps iOS simulator JPEG frames native for
fallback paths while converting larger Android PNG screenshots to scaled JPEG
frames. Use `format=native` for lowest server CPU or `format=jpeg` when
bandwidth is the bottleneck.
