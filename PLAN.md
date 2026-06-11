# PLAN.md: Rust Web App สำหรับควบคุม Android Emulator และ iOS Simulator

## Summary
สร้างโปรเจกต์ Rust full-stack ตั้งแต่ศูนย์ที่รันเป็น web application บนเครื่อง Mac และเปิดให้เครื่องใน LAN เข้าใช้งานได้โดยไม่ต้องใช้ token เพื่อควบคุม Android emulator และ iOS simulator ที่กำลังรันอยู่ในเครื่องเดียวกัน

แกนระบบ:
- Backend: Rust + Axum + Tokio
- Frontend: Rust-rendered web dashboard ใน crate `appmo-web`
- Transport: REST สำหรับคำสั่งทั่วไป และ WebSocket สำหรับ state/stream/progress
- Device control: ใช้ `adb` สำหรับ Android และ `xcrun simctl` สำหรับ iOS
- Security: v1 ใช้งานแบบ no-token สำหรับ local/LAN development

## Key Changes
- Rust workspace:
  - `appmo-server`: Axum server, auth middleware, REST/WebSocket API
  - `appmo-core`: device discovery, command model, process runner, error types
  - `appmo-web`: HTML/CSS/JS dashboard served by Rust
- Config:
  - `APPMO_BIND=0.0.0.0:8080`
  - `ANDROID_ADB_PATH=/Users/inteniquetic/Library/Android/sdk/platform-tools/adb`
  - `IOS_XCRUN_PATH=/usr/bin/xcrun`
- API v1:
  - `GET /health`
  - `GET /api/devices`
  - `GET /api/devices/:id/screenshot`
  - `POST /api/devices/:id/input/tap`
  - `POST /api/devices/:id/input/swipe`
  - `POST /api/devices/:id/input/text`
  - `POST /api/devices/:id/key`
  - `POST /api/devices/:id/app/install`
  - `POST /api/devices/:id/app/launch`
  - `POST /api/devices/:id/app/terminate`
  - `GET /api/devices/:id/logs`
  - `POST /api/devices/:id/record/start`
  - `POST /api/devices/:id/record/stop`
  - `WS /ws`

## Implementation Notes
- ทุก device command ผ่าน typed layer ใน `appmo-core` และรันผ่าน `tokio::process::Command` ด้วย args array
- `/health`, `/api/*`, และ `/ws` ไม่ต้องใช้ token
- Server start ได้โดยไม่ต้องตั้งค่า `APPMO_TOKEN`
- Screenshot stream v1 ใช้ polling จาก UI ทุก 1000ms แทน low-latency video stream
- Mouse/touch gestures บนภาพหน้าจอใช้ vendored `interact.js` 1.10.27 เพื่อไม่ต้องดูแล raw pointer edge cases เอง
- Android low-latency remote-control library ที่ควรต่อยอดคือ `scrcpy`/`@yume-chan/scrcpy`; v1 ยังใช้ `adb shell input` เป็น transport หลังจาก gesture layer
- iOS tap/swipe ใช้ capability path ของ `xcrun simctl io <udid> tap/swipe`; ถ้า Xcode ไม่รองรับจะคืน `UnsupportedCapability`

## Test Plan
- Unit tests:
  - parse `adb devices -l`
  - parse `simctl list devices --json`
  - command validation
- Integration tests:
  - mock process runner สำหรับ Android/iOS commands
  - verify REST routes return expected JSON/errors
- Manual verification:
  - start Android emulator แล้วเช็ก list, screenshot, tap, text, install, launch, logs
  - start iOS simulator แล้วเช็ก list, screenshot, install, launch, logs, record
  - เปิดจากเครื่องอื่นใน LAN ด้วย `http://<mac-lan-ip>:8080`
  - verify ว่า API และ WebSocket ใช้งานได้โดยไม่ต้องส่ง token

## Assumptions
- Repo เริ่มจากว่าง จึง scaffold ใหม่ได้เต็มรูปแบบ
- เครื่องนี้มี `adb` ที่ `/Users/inteniquetic/Library/Android/sdk/platform-tools/adb`
- เครื่องนี้มี `xcrun` ที่ `/usr/bin/xcrun`
- เป้าหมาย v1 คือ emulator/simulator ที่รันอยู่บนเครื่อง Mac เดียวกับ server
