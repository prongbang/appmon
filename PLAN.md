# PLAN.md: Rust Web App สำหรับควบคุม Android Emulator และ iOS Simulator

## Summary
สร้างโปรเจกต์ Rust full-stack ตั้งแต่ศูนย์ที่รันเป็น web application บนเครื่อง Mac และเปิดให้เครื่องใน LAN เข้าใช้งานได้โดยไม่ต้องใช้ token เพื่อควบคุม Android emulator และ iOS simulator ที่กำลังรันอยู่ในเครื่องเดียวกัน

แกนระบบ:
- Backend: Rust + Axum + Tokio
- Frontend: Rust-rendered web dashboard ใน crate `appmo-web`
- Transport: REST สำหรับคำสั่งทั่วไป และ WebSocket สำหรับ control/state/stream/progress
- UDP control: datagram JSON สำหรับ client/native tool ที่ไม่ใช่ browser; ค่าเริ่มต้น bind ที่พอร์ตถัดจาก `APPMO_BIND`
- Device control: ใช้ `adb` สำหรับ Android และ `xcrun simctl` สำหรับ iOS
- Security: v1 ใช้งานแบบ no-token สำหรับ local/LAN development

## Key Changes
- Rust workspace:
  - `appmo-server`: Axum server, auth middleware, REST/WebSocket API
  - `appmo-core`: device discovery, command model, process runner, error types
  - `appmo-web`: HTML/CSS/JS dashboard served by Rust
- Config:
  - `APPMO_BIND=0.0.0.0:8080`
  - `APPMO_UDP_BIND=0.0.0.0:8081` หรือ `off` เพื่อปิด UDP
  - `APPMO_AUTO_INSTALL_DEPS=false` เพื่อปิด dependency bootstrap ตอน server start
  - `ANDROID_ADB_PATH=/Users/inteniquetic/Library/Android/sdk/platform-tools/adb`
  - `IOS_XCRUN_PATH=/usr/bin/xcrun`
- API v1:
  - `GET /health`
  - `GET /api/devices`
  - `GET /api/devices/:id/screenshot`
  - `GET /api/devices/:id/screenshot-stream?format=native&fps=8`
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
  - `WS /ws` สำหรับ low-latency control message แบบ request/response

## Implementation Notes
- ทุก device command ผ่าน typed layer ใน `appmo-core`; คำสั่งทั่วไปยังรันผ่าน `tokio::process::Command` ด้วย args array
- Android tap/swipe/text/key ใช้ fast path เป็น persistent `adb -s <serial> shell` session ต่อ device แล้วเขียน `input ...` เข้า stdin เพื่อลด process-spawn latency
- iOS simulator screenshot ใช้ `xcrun simctl io <udid> screenshot --type=jpeg -` เพื่ออ่าน bytes ขนาดเล็กจาก stdout โดยตรงแทน temp file
- iOS Full Touch Control พอร์ตแนวทางจาก `ios-bridge`: ใช้ `idb describe` อ่าน point dimensions และใช้ `idb ui tap/swipe/text/key/button --udid <udid>` สำหรับ control แบบเต็ม
- ตอน server start จะตรวจและติดตั้ง tool ที่ขาดให้เองเท่าที่ทำได้: `android-platform-tools`, `idb-companion`, และ `fb-idb`; ถ้าปิดด้วย `APPMO_AUTO_INSTALL_DEPS=false` จะข้ามขั้นตอนนี้
- ถ้า `idb` ไม่พร้อม iOS tap fallback เป็น `osascript` + macOS Accessibility map พิกัดจาก screenshot ไปยัง Simulator window; swipe/text/key ต้องใช้ `idb`
- Web UI ส่ง control ผ่าน WebSocket ก่อน และ fallback ไป REST endpoint เดิมถ้า WebSocket ไม่พร้อม
- Browser ใช้ UDP raw โดยตรงไม่ได้ จึงยังใช้ WebSocket สำหรับหน้าเว็บ; UDP protocol มีไว้สำหรับ native/local control client ที่ส่ง datagram ได้
- `/health`, `/api/*`, และ `/ws` ไม่ต้องใช้ token
- Server start ได้โดยไม่ต้องตั้งค่า `APPMO_TOKEN`
- Screen preview default ใช้ one-shot screenshot + adaptive polling ที่ปรับ FPS ได้, ไม่ปล่อย fetch ซ้อน, preload frame ก่อน swap และ revoke object URL เก่าเพื่อลด memory/paint jitter
- Experimental screenshot stream ใช้ Rust-served multipart stream (`multipart/x-mixed-replace`) และปรับ `fps`, `format`, `max_width`, และ JPEG `quality` ได้; server คุม cadence โดยหักเวลาที่ใช้ capture ออกจาก frame delay
- Mouse/touch gestures บนภาพหน้าจอใช้ vendored `interact.js` 1.10.27 เพื่อไม่ต้องดูแล raw pointer edge cases เอง
- Android low-latency remote-control library ที่ควรต่อยอดคือ `scrcpy`/`@yume-chan/scrcpy`; v1 ใช้ persistent `adb shell input` เป็น transport หลังจาก gesture layer
- Xcode เครื่องนี้ไม่มี `simctl io tap/swipe/key`; iOS full touch ใช้ `idb` แทน `simctl`

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
