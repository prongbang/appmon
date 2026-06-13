# Appmon

Appmon คือ web console สำหรับควบคุม Android emulator และ iOS simulator ที่รันอยู่บน Mac เครื่องเดียวกัน รองรับการดู preview หน้าจอ, ส่ง touch/keyboard control, start/stop device, install/launch/stop app และเปิดใช้งานผ่าน localhost หรือ LAN ได้

## Requirements

- macOS
- Rust toolchain
- Android SDK / Android Emulator / `adb` สำหรับ Android
- Xcode command line tools / `xcrun` สำหรับ iOS simulator
- Homebrew แนะนำให้มี เพื่อให้ Appmon auto-install tools ที่ขาดได้ง่ายขึ้น

ตอน start server Appmon จะตรวจ dependency ที่จำเป็นและพยายามติดตั้งให้อัตโนมัติเมื่อทำได้ เช่น Android platform tools, `idb-companion`, `fb-idb` และ `ffmpeg` สำหรับ WebRTC preview

## Install

```sh
cd /path/to/appmon
cp .env.example .env
cargo check
```

ถ้าต้องการปรับ path ของ tools ให้แก้ไฟล์ `.env`

```env
APPMON_BIND=0.0.0.0:8080
ANDROID_ADB_PATH=/Users/<user>/Library/Android/sdk/platform-tools/adb
IOS_XCRUN_PATH=/usr/bin/xcrun
```

## Configuration

| Variable | Description |
| --- | --- |
| `APPMON_BIND` | host/port ของ web server เช่น `127.0.0.1:18080` หรือ `0.0.0.0:18080` |
| `APPMON_UDP_BIND` | host/port ของ UDP control ถ้าไม่ตั้งค่าจะใช้ port ถัดจาก `APPMON_BIND` |
| `APPMON_AUTO_INSTALL_DEPS` | ตั้งเป็น `false` เพื่อปิด auto-install dependency |
| `ANDROID_ADB_PATH` | path ของ `adb` |
| `ANDROID_EMULATOR_PATH` | path ของ Android emulator binary |
| `IOS_XCRUN_PATH` | path ของ `xcrun` |
| `APPMON_IDB_PATH` | path ของ `idb` สำหรับ iOS touch control แบบเต็ม |
| `APPMON_OSASCRIPT_PATH` | path ของ `osascript` สำหรับ fallback control บน iOS simulator |

ตัวแปรเก่ากลุ่ม `APPMO_*` ยังรองรับเป็น fallback เพื่อ compatibility แต่แนะนำให้ใช้ `APPMON_*`

## Run Server

รันแบบ localhost:

```sh
APPMON_BIND=127.0.0.1:18080 cargo run -p appmon-server
```

เปิดใน browser:

```text
http://127.0.0.1:18080
```

รันให้เครื่องอื่นในวง LAN เปิดได้:

```sh
APPMON_BIND=0.0.0.0:18080 cargo run -p appmon-server
```

จากเครื่องอื่นให้เปิดด้วย IP ของ Mac:

```text
http://<mac-lan-ip>:18080
```

หมายเหตุ: web UI และ API ไม่มี authentication เพราะออกแบบสำหรับ local/LAN development เท่านั้น

## Usage

1. เปิด Android emulator หรือ iOS simulator ที่ต้องการใช้งาน
2. เปิด Appmon web UI
3. กด refresh ในหน้า Devices เพื่อโหลดรายการ device ล่าสุด
4. ใช้ปุ่ม `Start` / `Stop` ในแต่ละ device เพื่อ boot หรือ shutdown
5. เลือก device ที่ต้องการ monitor
6. ใช้หน้า Monitor เพื่อดู preview และส่ง tap/swipe/key input
7. ใช้ปุ่ม App เพื่อ install, launch หรือ stop app บน device ที่เลือก

รายการ device จะแยกกลุ่ม Android และ iOS และแสดงเฉพาะข้อมูลที่จำเป็น เช่นชื่อ device, platform และสถานะ

## Preview And Control

Appmon มีหลายโหมดสำหรับ preview หน้าจอ:

- `WebRTC`: โหมด latency ต่ำสุด ใช้ VP8 media track เพื่อให้ browser decode video ด้วย native decoder
- `Stream`: fallback แบบ multipart stream
- `Polling`: fallback แบบ screenshot polling ที่ conservative กว่า

Android control ใช้ persistent `adb shell` session เพื่อลด overhead จากการ spawn process ทุกครั้ง ส่วน iOS touch control ใช้ `idb` เพื่อ map coordinate จาก browser ไปเป็น iOS point coordinate ได้แม่นยำกว่า

ถ้า `idb` ยังไม่พร้อม iOS tap จะ fallback ไปที่ Simulator-window AppleScript control แต่ swipe/text/key ต้องใช้ `idb` เพื่อให้ทำงานครบ

## App Controls

ใน panel App:

- ใส่ package name หรือ bundle id เช่น `com.example.app`
- ใส่ path ของไฟล์ install เช่น `/path/to/app.apk` หรือ `/path/to/App.app`
- กด `Install` เพื่อติดตั้ง app
- กด `Launch` เพื่อเปิด app
- กด `Stop` เพื่อหยุด app

## UDP Control

ถ้า `APPMON_BIND=127.0.0.1:18081` ค่า UDP control default จะอยู่ที่ `127.0.0.1:18082`

ตัวอย่างส่ง tap ผ่าน UDP:

```sh
printf '{"request_id":"tap-1","device_id":"android:emulator-5554","type":"tap","x":320,"y":640}' \
  | nc -u -w1 127.0.0.1 18082
```

ปิด UDP control ได้ด้วย:

```sh
APPMON_UDP_BIND=off cargo run -p appmon-server
```

## Troubleshooting

ถ้าเปิดผ่าน IP ไม่ได้ ให้รัน server ด้วย `APPMON_BIND=0.0.0.0:<port>` ตรวจว่า Mac และอุปกรณ์อยู่ Wi-Fi เดียวกัน และดูว่า firewall ไม่ได้ block port นั้น

ถ้า device ไม่ขึ้น ให้ตรวจว่า emulator/simulator เปิดอยู่ แล้วกด refresh ในหน้า Devices สำหรับ Android ตรวจด้วย `adb devices` และสำหรับ iOS ตรวจด้วย `xcrun simctl list devices`

ถ้า iOS touch/swipe/text ทำงานไม่ครบ ให้ติดตั้ง `idb` หรือกำหนด `APPMON_IDB_PATH` ให้ถูกต้อง

ถ้า port ถูกใช้อยู่ ให้เปลี่ยน port เช่น:

```sh
APPMON_BIND=127.0.0.1:18081 cargo run -p appmon-server
```

ถ้าไม่ต้องการให้ Appmon auto-install dependency ตอน start:

```sh
APPMON_AUTO_INSTALL_DEPS=false cargo run -p appmon-server
```
