# Appmon

Appmon is a local web console for controlling Android emulators and iOS simulators on a Mac. It can start and stop devices, show a live screen preview, send touch and keyboard input, and install, launch, or stop apps from one browser UI.

## Features

- Android and iOS device list grouped by platform
- Start / stop device actions with status checks
- Low-latency preview with WebRTC when available
- Touch, swipe, text, key, and button control
- App install / launch / stop controls
- Localhost and LAN access for development
- UDP control endpoint for external automation

## Requirements

- macOS
- Rust toolchain
- Xcode command line tools for `xcrun`
- Android SDK / Android Emulator / `adb`
- Homebrew, recommended for automatic dependency setup

Appmon checks local tools when the server starts. When possible, it can auto-install missing developer dependencies such as Android platform tools, `idb-companion`, `fb-idb`, and `ffmpeg`.

## Quick Install

For the simplest local setup on macOS:

```sh
xcode-select --install
brew install rust android-platform-tools ffmpeg
git clone https://github.com/prongbang/appmon.git
cd appmon
cp .env.example .env
APPMON_BIND=127.0.0.1:18080 cargo run -p appmon-server
```

Then open:

```text
http://127.0.0.1:18080
```

If you already have Rust, Xcode command line tools, and Android platform tools installed, the shortest path is:

```sh
git clone https://github.com/prongbang/appmon.git
cd appmon
cp .env.example .env
cargo run -p appmon-server
```

Appmon starts a local web UI and checks required tools on startup. With Homebrew available, it can also try to install missing runtime dependencies automatically.

## Install Details

```sh
git clone https://github.com/prongbang/appmon.git
cd appmon
cp .env.example .env
cargo check
```

Edit `.env` if your local tool paths are different:

```env
APPMON_BIND=0.0.0.0:8080
ANDROID_ADB_PATH=/Users/<user>/Library/Android/sdk/platform-tools/adb
IOS_XCRUN_PATH=/usr/bin/xcrun
```

## Run

Run for localhost only:

```sh
APPMON_BIND=127.0.0.1:18080 cargo run -p appmon-server
```

Open:

```text
http://127.0.0.1:18080
```

Run for LAN access:

```sh
APPMON_BIND=0.0.0.0:18080 cargo run -p appmon-server
```

Open from another device on the same network:

```text
http://<mac-lan-ip>:18080
```

Health check:

```sh
curl http://127.0.0.1:18080/health
```

The web UI and API are intentionally unauthenticated and should only be used on trusted local or LAN networks.

## Usage

1. Start the Android emulator or iOS simulator you want to control.
2. Open the Appmon web UI.
3. Refresh the device list.
4. Use `Start` or `Stop` on a device row when needed.
5. Select a device to open the monitor.
6. Use the live preview to tap, swipe, type, or send keys.
7. Use the App panel to install, launch, or stop an app.

## App Controls

Use package name or bundle id for launch and stop:

```text
com.example.app
```

Use a local app path for install:

```text
/path/to/app.apk
/path/to/App.app
```

## Preview Modes

Appmon supports multiple preview paths:

- `WebRTC`: lowest-latency mode. Android uses the emulator native gRPC/WebRTC `RtcService` when `APPMON_ANDROID_GRPC_ENDPOINT` is set, then falls back to Appmon's VP8 media track
- `Stream`: multipart stream fallback
- `Polling`: conservative screenshot polling fallback

Android input can use the Android Emulator gRPC controller when `APPMON_ANDROID_GRPC_ENDPOINT` is set, matching the same direct `sendTouch` / `sendKey` control surface used by Google's WebRTC emulator sample. If gRPC is unavailable, Appmon falls back to a persistent `adb shell` session and streams pointer gestures as `motionevent` down/move/up commands so drag and scroll feedback does not wait for pointer release. iOS touch control uses `idb` for accurate simulator coordinates. If `idb` is unavailable, tap can fall back to Simulator-window AppleScript control, but swipe, text, and key control require `idb`.

## Configuration

| Variable | Description |
| --- | --- |
| `APPMON_BIND` | Web server bind address, for example `127.0.0.1:18080` or `0.0.0.0:18080` |
| `APPMON_UDP_BIND` | UDP control bind address. Defaults to the next port after `APPMON_BIND` |
| `APPMON_AUTO_INSTALL_DEPS` | Set to `false` to disable automatic dependency installation |
| `ANDROID_ADB_PATH` | Path to `adb` |
| `ANDROID_EMULATOR_PATH` | Path to the Android emulator binary |
| `APPMON_ANDROID_GRPC_ENDPOINT` | Optional Android Emulator gRPC endpoint, for example `http://127.0.0.1:8554`. When set, Android Start adds `-grpc <port>` |
| `IOS_XCRUN_PATH` | Path to `xcrun` |
| `APPMON_IDB_PATH` | Path to `idb` for full iOS simulator touch control |
| `APPMON_OSASCRIPT_PATH` | Path to `osascript` for iOS fallback control |

Legacy `APPMO_*` variables are still accepted as fallbacks, but new setup should use `APPMON_*`.

Disable automatic dependency installation:

```sh
APPMON_AUTO_INSTALL_DEPS=false cargo run -p appmon-server
```

## UDP Control

If `APPMON_BIND=127.0.0.1:18081`, UDP control defaults to `127.0.0.1:18082`.

Example tap command:

```sh
printf '{"request_id":"tap-1","device_id":"android:emulator-5554","type":"tap","x":320,"y":640}' \
  | nc -u -w1 127.0.0.1 18082
```

Disable UDP control:

```sh
APPMON_UDP_BIND=off cargo run -p appmon-server
```

## Troubleshooting

If the UI cannot be opened from another device, run with `APPMON_BIND=0.0.0.0:<port>`, make sure both devices are on the same network, and check the macOS firewall.

If Android devices do not appear, check:

```sh
adb devices
```

If iOS simulators do not appear, check:

```sh
xcrun simctl list devices
```

If iOS touch, swipe, text, or key control is incomplete, install `idb` or set `APPMON_IDB_PATH`.

If the port is already in use, choose another port:

```sh
APPMON_BIND=127.0.0.1:18081 cargo run -p appmon-server
```
