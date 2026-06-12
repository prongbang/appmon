use dioxus::prelude::*;

const INTERACT_JS: &str = include_str!("../assets/interact.min.js");

pub fn dashboard_html() -> String {
    let app = dioxus_ssr::render_element(rsx! { App {} });
    let mut html = String::from(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Appmo</title>
  <style>"#,
    );
    html.push_str(TAILWIND_THEME_CSS);
    html.push_str(
        r#"</style>
</head>
<body>"#,
    );
    html.push_str(&app);
    html.push_str("<script>");
    html.push_str(INTERACT_JS);
    html.push_str("</script><script>");
    html.push_str(APP_SCRIPT);
    html.push_str(
        r#"</script>
</body>
</html>"#,
    );
    html
}

#[component]
fn App() -> Element {
    rsx! {
        div { class: "min-h-screen app-shell text-slate-950",
            main { class: "app-layout",
                div { class: "workspace-rail",
                    div { class: "brand-lockup",
                        div { class: "brand-mark", "A" }
                        div {
                            h1 { class: "brand-title", "Appmo" }
                            p { class: "brand-subtitle", "Device control console" }
                        }
                    }
                    DevicesPane {}
                }
                MonitorPane {}
                AppControls {}
            }
        }
    }
}

#[component]
fn DevicesPane() -> Element {
    rsx! {
        aside { class: "devices-pane panel-section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Devices" }
                    p { class: "section-kicker", "Android and iOS targets" }
                }
                button { id: "refresh", class: "icon-btn", title: "Refresh devices", aria_label: "Refresh devices",
                    span { class: "refresh-icon" }
                }
            }
            div { id: "devices", class: "device-list" }
        }
    }
}

#[component]
fn MonitorPane() -> Element {
    rsx! {
        section { class: "monitor-pane",
            div { class: "monitor-card",
                div { class: "monitor-toolbar",
                    div {
                        h2 { class: "monitor-title", "Monitor" }
                        span { id: "selectedMeta", class: "text-xs text-slate-500", "Select a device to begin" }
                    }
                    div { class: "toolbar-actions",
                        span { id: "statusChip", class: "status-pill", "Idle" }
                        button { id: "fullscreenToggle", class: "icon-btn", title: "Enter fullscreen", aria_label: "Enter fullscreen",
                            span { class: "fullscreen-icon" }
                        }
                        button { id: "settingsOpen", class: "btn btn-secondary", "Preview" }
                    }
                }
                div { id: "screenWrap", class: "screen-wrap",
                    img { id: "screen", alt: "Device screenshot" }
                    div { id: "screenEmpty", class: "empty-screen", "Select a device" }
                }
                DeviceNav {}
                p { id: "status", class: "mt-3 min-h-5 text-xs text-slate-700" }
            }

            PreviewSettingsModal {}

            LogsPanel {}
        }
    }
}

#[component]
fn PreviewSettingsModal() -> Element {
    rsx! {
        div { id: "settingsModal", class: "settings-modal", aria_hidden: "true",
            div { class: "settings-dialog", role: "dialog", aria_modal: "true", aria_labelledby: "settingsTitle",
                div { class: "section-head compact",
                    div {
                        h2 { id: "settingsTitle", class: "section-title", "Preview Controls" }
                        p { class: "section-kicker", "Stream, capture, and diagnostics" }
                    }
                    button { id: "settingsClose", class: "icon-btn", title: "Close", aria_label: "Close",
                        span { class: "close-icon" }
                    }
                }
                div { class: "settings-grid",
                    select { id: "viewMode", title: "Preview mode", aria_label: "Preview mode",
                        option { value: "poll", selected: true, "Polling" }
                        option { value: "webrtc", "WebRTC" }
                        option { value: "stream", "Stream" }
                    }
                    select { id: "pollFps", title: "Polling FPS", aria_label: "Polling FPS",
                        option { value: "1", "1 fps" }
                        option { value: "2", "2 fps" }
                        option { value: "4", selected: true, "4 fps" }
                        option { value: "6", "6 fps" }
                        option { value: "8", "8 fps" }
                    }
                    select { id: "streamFps", title: "Stream FPS", aria_label: "Stream FPS",
                        option { value: "4", "4 fps" }
                        option { value: "8", selected: true, "8 fps" }
                        option { value: "12", "12 fps" }
                        option { value: "15", "15 fps" }
                    }
                    select { id: "streamFormat", title: "Stream format", aria_label: "Stream format",
                        option { value: "auto", selected: true, "Auto smooth" }
                        option { value: "native", "Fast native" }
                        option { value: "jpeg", "Small JPEG" }
                    }
                    select { id: "streamScale", title: "Stream scale", aria_label: "Stream scale",
                        option { value: "540", "540p" }
                        option { value: "720", selected: true, "720p" }
                        option { value: "1080", "1080p" }
                        option { value: "4096", "Full" }
                    }
                    select { id: "streamQuality", title: "Stream quality", aria_label: "Stream quality",
                        option { value: "55", "Eco" }
                        option { value: "70", selected: true, "Balanced" }
                        option { value: "85", "Sharp" }
                    }
                }
                div { class: "settings-actions",
                    button { id: "shot", class: "btn btn-secondary", "Screenshot" }
                    button { id: "logsBtn", class: "btn btn-secondary", "Logs" }
                    button { id: "recordStart", class: "btn btn-secondary", "Record" }
                    button { id: "recordStop", class: "btn btn-danger", "Stop" }
                }
            }
        }
    }
}

#[component]
fn DeviceNav() -> Element {
    rsx! {
        nav { id: "deviceNav", class: "device-nav", aria_label: "Device navigation", "data-platform": "none",
            button { id: "navBack", title: "Back", aria_label: "Back",
                span { class: "nav-back" }
            }
            button { id: "navHome", title: "Home", aria_label: "Home",
                span { class: "nav-home" }
            }
            button { id: "navRecents", title: "Recents", aria_label: "Recents",
                span { class: "nav-recents" }
            }
        }
    }
}

#[component]
fn LogsPanel() -> Element {
    rsx! {
        div { class: "logs-panel",
            div { class: "section-head compact",
                div {
                    h2 { class: "section-title", "Logs" }
                    p { class: "section-kicker", "Last 300 lines" }
                }
            }
            pre { id: "logs" }
        }
    }
}

#[component]
fn AppControls() -> Element {
    rsx! {
        aside { class: "app-controls panel-section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "App" }
                    p { class: "section-kicker", "Install and launch" }
                }
            }
            div { class: "field-band",
                input { id: "appId", placeholder: "Package or bundle id" }
                input { id: "appPath", placeholder: "/path/to .apk or .app" }
                div { class: "action-grid",
                    button { id: "install", class: "btn btn-secondary", "Install" }
                    button { id: "launch", class: "btn", "Launch" }
                    button { id: "terminate", class: "btn btn-secondary", "Stop" }
                }
            }
        }
    }
}

const TAILWIND_THEME_CSS: &str = r#"
:root {
  color-scheme: light;
  --tw-slate-50: #f8fafc;
  --tw-slate-100: #f1f5f9;
  --tw-slate-200: #e2e8f0;
  --tw-slate-300: #cbd5e1;
  --tw-slate-500: #64748b;
  --tw-slate-700: #334155;
  --tw-slate-900: #0f172a;
  --tw-slate-950: #020617;
  --tw-coral-50: #fff4f1;
  --tw-coral-100: #ffe2dc;
  --tw-coral-500: #f05d4f;
  --tw-coral-600: #d94b3f;
  --tw-teal-500: #15a6a3;
  --tw-amber-400: #f5b84b;
  --tw-white: #ffffff;
  --theme-line: rgba(203, 213, 225, .82);
  --theme-soft-line: rgba(226, 232, 240, .74);
  --theme-ring: rgba(21, 166, 163, .24);
  --theme-radius: 8px;
  --shadow-panel: 0 20px 48px rgba(15, 23, 42, .08);
  --shadow-soft: 0 12px 28px rgba(15, 23, 42, .06);
}
* { box-sizing: border-box; }
body {
  margin: 0;
  min-height: 100vh;
  font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #eef3f8;
  color: var(--tw-slate-950);
}
button, input, textarea, select { font: inherit; min-width: 0; }
button { cursor: pointer; white-space: nowrap; }
input, textarea, select {
  width: 100%;
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  padding: 10px 12px;
  background: var(--tw-white);
  color: var(--tw-slate-950);
  box-shadow: inset 0 1px 0 rgba(255,255,255,.8);
}
input:focus, textarea:focus, select:focus, button:focus-visible {
  outline: 3px solid var(--theme-ring);
  outline-offset: 1px;
}
select {
  width: auto;
  min-width: 104px;
  cursor: pointer;
  min-height: 40px;
}
pre {
  margin: 0;
  min-height: 144px;
  max-height: 260px;
  overflow: auto;
  white-space: pre-wrap;
  border: 1px solid rgba(15, 23, 42, .82);
  border-radius: var(--theme-radius);
  padding: 12px;
  background: var(--tw-slate-950);
  color: var(--tw-slate-100);
  font-size: 12px;
  line-height: 1.55;
}
.min-h-screen { min-height: 100vh; }
.min-h-5 { min-height: 1.25rem; }
.grid { display: grid; }
.grid-cols-app { grid-template-columns: 280px minmax(0, 1fr) 320px; }
.grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.flex { display: flex; }
.flex-wrap { flex-wrap: wrap; }
.items-center { align-items: center; }
.justify-between { justify-content: space-between; }
.justify-end { justify-content: flex-end; }
.gap-2 { gap: .5rem; }
.gap-3 { gap: .75rem; }
.p-5 { padding: 1.25rem; }
.p-6 { padding: 1.5rem; }
.mb-4 { margin-bottom: 1rem; }
.mb-5 { margin-bottom: 1.25rem; }
.mt-3 { margin-top: .75rem; }
.mt-5 { margin-top: 1.25rem; }
.overflow-auto { overflow: auto; }
.border-r { border-right-width: 1px; border-right-style: solid; }
.border-l { border-left-width: 1px; border-left-style: solid; }
.border-slate-200 { border-color: var(--theme-line); }
.bg-coral { background: var(--tw-coral-500); }
.bg-slate-50 { background: var(--tw-slate-50); }
.bg-white\/90 { background: rgba(255, 255, 255, .9); }
.bg-white\/95 { background: rgba(255, 255, 255, .95); }
.text-slate-950 { color: var(--tw-slate-950); }
.text-slate-700 { color: var(--tw-slate-700); }
.text-slate-500 { color: var(--tw-slate-500); }
.text-base { font-size: 1rem; }
.text-2xl { font-size: 1.5rem; line-height: 2rem; }
.text-xs { font-size: .75rem; line-height: 1rem; }
.font-bold { font-weight: 780; }
.font-extrabold { font-weight: 850; }

.app-shell {
  min-height: 100vh;
  background:
    linear-gradient(135deg, rgba(21, 166, 163, .12), transparent 34%),
    linear-gradient(315deg, rgba(240, 93, 79, .12), transparent 38%),
    #eef3f8;
}
.app-layout {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 292px minmax(0, 1fr) 320px;
  gap: 14px;
  padding: 14px;
}
.workspace-rail,
.monitor-pane,
.app-controls {
  border-radius: var(--theme-radius);
  background: var(--tw-white);
  border: 1px solid var(--theme-soft-line);
  box-shadow: var(--shadow-panel);
}
.workspace-rail {
  min-height: calc(100vh - 28px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.brand-lockup {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 18px;
  border-bottom: 1px solid var(--theme-soft-line);
  background: linear-gradient(180deg, #ffffff, #f8fbfc);
}
.brand-mark {
  width: 38px;
  height: 38px;
  border-radius: var(--theme-radius);
  display: grid;
  place-items: center;
  color: #fff;
  font-weight: 850;
  background: linear-gradient(135deg, var(--tw-slate-950), var(--tw-teal-500));
  box-shadow: 0 12px 24px rgba(15, 23, 42, .16);
}
.brand-title,
.brand-subtitle,
.section-title,
.section-kicker {
  margin: 0;
}
.brand-title {
  font-size: 18px;
  line-height: 1.15;
  font-weight: 850;
}
.brand-subtitle,
.section-kicker {
  color: var(--tw-slate-500);
  font-size: 12px;
  line-height: 1.35;
}
.panel-section {
  padding: 16px;
}
.devices-pane {
  min-height: 0;
  flex: 1;
  overflow: auto;
}
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}
.section-head.compact { margin-bottom: 10px; }
.section-title {
  color: var(--tw-slate-950);
  font-size: 15px;
  line-height: 1.2;
  font-weight: 820;
}
.icon-btn {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  background: var(--tw-white);
  color: var(--tw-slate-700);
  box-shadow: var(--shadow-soft);
}
.refresh-icon {
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 999px;
  position: relative;
}
.refresh-icon::after {
  content: "";
  position: absolute;
  right: -2px;
  top: -5px;
  width: 7px;
  height: 7px;
  border-top: 2px solid currentColor;
  border-right: 2px solid currentColor;
  transform: rotate(22deg);
}
.close-icon {
  width: 16px;
  height: 16px;
  position: relative;
}
.close-icon::before,
.close-icon::after {
  content: "";
  position: absolute;
  left: 7px;
  top: 1px;
  width: 2px;
  height: 14px;
  border-radius: 999px;
  background: currentColor;
}
.close-icon::before { transform: rotate(45deg); }
.close-icon::after { transform: rotate(-45deg); }
.fullscreen-icon {
  width: 17px;
  height: 17px;
  position: relative;
}
.fullscreen-icon::before,
.fullscreen-icon::after {
  content: "";
  position: absolute;
  width: 7px;
  height: 7px;
  border-color: currentColor;
  border-style: solid;
}
.fullscreen-icon::before {
  top: 0;
  left: 0;
  border-width: 2px 0 0 2px;
}
.fullscreen-icon::after {
  right: 0;
  bottom: 0;
  border-width: 0 2px 2px 0;
}
.screen-wrap:fullscreen,
.screen-wrap:-webkit-full-screen {
  width: 100vw;
  height: 100vh;
  border: 0;
  border-radius: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #090d14;
}
.screen-wrap:fullscreen #screen,
.screen-wrap:-webkit-full-screen #screen {
  max-width: 100%;
  max-height: 100%;
}
.monitor-pane {
  min-width: 0;
  min-height: calc(100vh - 28px);
  overflow: auto;
  padding: 16px;
  background: rgba(255, 255, 255, .74);
  backdrop-filter: blur(18px);
}
.monitor-card,
.logs-panel {
  border-radius: var(--theme-radius);
  background: var(--tw-white);
  box-shadow: var(--shadow-panel);
  border: 1px solid var(--theme-soft-line);
}
.monitor-card {
  padding: 14px;
}
.monitor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  margin-bottom: 12px;
}
.monitor-title {
  margin: 0 0 2px;
  font-size: 22px;
  line-height: 1.18;
  font-weight: 850;
}
.toolbar-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
}
.settings-modal {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: none;
  align-items: flex-start;
  justify-content: flex-end;
  padding: 22px;
  background: rgba(15, 23, 42, .34);
  backdrop-filter: blur(8px);
}
.settings-modal.open { display: flex; }
.settings-dialog {
  width: min(340px, calc(100vw - 44px));
  max-height: calc(100vh - 44px);
  overflow: auto;
  border: 1px solid var(--theme-soft-line);
  border-radius: var(--theme-radius);
  background: rgba(248, 250, 252, .98);
  box-shadow: 0 28px 70px rgba(15, 23, 42, .22);
  padding: 16px;
}
.settings-grid {
  display: grid;
  gap: 10px;
}
.settings-grid select,
.settings-actions .btn {
  width: 100%;
}
.settings-actions {
  display: grid;
  gap: 10px;
  margin-top: 14px;
}
.logs-panel {
  margin-top: 14px;
  padding: 14px;
}
.btn {
  border: 1px solid transparent;
  border-radius: var(--theme-radius);
  padding: 10px 13px;
  min-height: 40px;
  background: var(--tw-teal-500);
  color: var(--tw-white);
  font-weight: 760;
  box-shadow: 0 10px 20px rgba(21, 166, 163, .18);
  transition: transform .14s ease, border-color .14s ease, background .14s ease, box-shadow .14s ease;
}
.btn:hover,
.icon-btn:hover,
.device:hover { transform: translateY(-1px); }
.icon-btn.active {
  border-color: rgba(21, 166, 163, .58);
  color: #0f766e;
  background: #effdfb;
}
.icon-btn.active .fullscreen-icon::before {
  top: 2px;
  left: 2px;
  border-width: 0 2px 2px 0;
}
.icon-btn.active .fullscreen-icon::after {
  right: 2px;
  bottom: 2px;
  border-width: 2px 0 0 2px;
}
.btn-secondary {
  background: var(--tw-white);
  border-color: var(--theme-line);
  color: var(--tw-slate-900);
  box-shadow: var(--shadow-soft);
}
.btn-danger {
  background: var(--tw-coral-600);
  border-color: var(--tw-coral-600);
  color: var(--tw-white);
  box-shadow: 0 10px 20px rgba(217, 75, 63, .18);
}
.status-pill {
  border: 1px solid rgba(21, 166, 163, .22);
  border-radius: var(--theme-radius);
  padding: 7px 10px;
  background: rgba(21, 166, 163, .08);
  color: #0f766e;
  font-size: 12px;
  font-weight: 750;
  white-space: nowrap;
}
.device-list {
  display: grid;
  gap: 10px;
}
.device {
  width: 100%;
  min-height: 82px;
  text-align: left;
  color: var(--tw-slate-950);
  background: var(--tw-white);
  border: 1px solid var(--theme-soft-line);
  border-radius: var(--theme-radius);
  padding: 13px;
  display: grid;
  gap: 5px;
  box-shadow: var(--shadow-soft);
  transition: transform .14s ease, border-color .14s ease, background .14s ease;
}
.device strong { font-size: 15px; }
.device.active {
  border-color: rgba(21, 166, 163, .58);
  outline: 3px solid var(--theme-ring);
  background: #effdfb;
}
.muted { color: var(--tw-slate-500); font-size: 12px; overflow-wrap: anywhere; }
.screen-wrap {
  display: grid;
  place-items: center;
  min-height: min(66vh, 620px);
  border-radius: var(--theme-radius) var(--theme-radius) 0 0;
  background:
    radial-gradient(circle at 50% 35%, rgba(148, 163, 184, .18), transparent 35%),
    #090d14;
  overflow: hidden;
  touch-action: none;
  position: relative;
  border: 1px solid rgba(15, 23, 42, .92);
  border-bottom: 0;
}
#screen {
  max-width: 100%;
  max-height: min(66vh, 620px);
  object-fit: contain;
  display: none;
  cursor: crosshair;
  user-select: none;
  -webkit-user-drag: none;
  image-rendering: auto;
  transform: translateZ(0);
  will-change: contents;
}
.empty-screen {
  color: rgba(255,255,255,.72);
  font-weight: 720;
  text-align: center;
  padding: 28px;
}
.device-nav {
  height: 44px;
  border-radius: 0 0 var(--theme-radius) var(--theme-radius);
  border-top: 1px solid rgba(148, 163, 184, .22);
  background: #171b22;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  align-items: center;
  overflow: hidden;
}
.device-nav[data-platform="ios"] {
  grid-template-columns: 1fr;
}
.device-nav[data-platform="ios"] #navBack,
.device-nav[data-platform="ios"] #navRecents {
  display: none;
}
.device-nav .nav-hidden {
  display: none;
}
.device-nav button {
  height: 100%;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: #d1d5db;
  display: grid;
  place-items: center;
  padding: 0;
}
.device-nav button:hover { background: rgba(255,255,255,.08); }
.nav-back {
  width: 0;
  height: 0;
  border-top: 9px solid transparent;
  border-bottom: 9px solid transparent;
  border-right: 14px solid currentColor;
}
.nav-home {
  width: 20px;
  height: 20px;
  border: 2px solid currentColor;
  border-radius: 999px;
}
.nav-recents {
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-radius: 2px;
}
.field-band {
  border: 1px solid var(--theme-soft-line);
  border-radius: var(--theme-radius);
  background: rgba(248, 250, 252, .78);
  padding: 14px;
  display: grid;
  gap: 10px;
}
.action-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
@media (max-width: 1240px) {
  .app-layout { grid-template-columns: 280px minmax(0, 1fr); }
  .app-controls { grid-column: 1 / -1; min-height: auto; }
  .field-band { grid-template-columns: minmax(0, 1fr) minmax(0, 1.25fr) auto; align-items: center; }
  .action-grid { min-width: 290px; }
}
@media (max-width: 920px) {
  .app-layout {
    grid-template-columns: minmax(0, 1fr);
    padding: 10px;
    gap: 10px;
  }
  .monitor-pane {
    order: -1;
    min-height: auto;
    padding: 12px;
  }
  .workspace-rail { min-height: auto; }
  .devices-pane { max-height: 360px; }
  .screen-wrap { min-height: 390px; }
  .field-band { grid-template-columns: 1fr; }
  .action-grid { min-width: 0; }
}
@media (max-width: 560px) {
  body { background: #f3f6fa; }
  .app-layout { padding: 0; gap: 0; }
  .workspace-rail,
  .monitor-pane,
  .app-controls {
    border-radius: 0;
    border-left: 0;
    border-right: 0;
    box-shadow: none;
  }
  .monitor-toolbar,
  .section-head {
    align-items: flex-start;
  }
  .monitor-toolbar {
    display: grid;
  }
  .toolbar-actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    width: 100%;
  }
  .toolbar-actions .status-pill {
    grid-column: 1 / -1;
    text-align: center;
  }
  .toolbar-actions .btn {
    width: 100%;
  }
  .toolbar-actions .icon-btn {
    width: 100%;
  }
  .settings-modal {
    align-items: flex-end;
    justify-content: center;
    padding: 10px;
  }
  .settings-dialog {
    width: 100%;
    max-height: calc(100vh - 20px);
  }
  .screen-wrap {
    min-height: 58vh;
    border-radius: var(--theme-radius) var(--theme-radius) 0 0;
  }
  #screen { max-height: 58vh; }
  .action-grid { grid-template-columns: 1fr; }
  .flex-wrap, .grid-cols-3 { grid-template-columns: 1fr; display: grid; }
}
"#;

const APP_SCRIPT: &str = r#"
const state = {
  devices: [],
  selected: null,
  ws: null,
  poll: null,
  pollAbort: null,
  streamAbort: null,
  webrtcPeer: null,
  webrtcChannel: null,
  webrtcSession: null,
  webrtcFrames: new Map(),
  previewUrl: null,
  previewSeq: 0,
  pointerStart: null,
  pending: new Map(),
  requestSeq: 0
};
const el = id => document.getElementById(id);

function setStatus(text) {
  el('status').textContent = text || '';
  el('statusChip').textContent = text ? 'Active' : 'Idle';
}
function headers() { return { 'content-type': 'application/json' }; }
function selectedId() {
  if (!state.selected) throw new Error('Select a device first');
  return encodeURIComponent(state.selected.id);
}
async function api(path, options = {}) {
  const res = await fetch(path, { ...options, headers: { ...headers(), ...(options.headers || {}) } });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || `${res.status} ${res.statusText}`);
  }
  return res;
}
async function json(path, options) {
  return api(path, options).then(r => r.json());
}
function renderDevices() {
  el('devices').innerHTML = '';
  el('selectedMeta').textContent = state.selected
    ? `${state.selected.name} / ${state.selected.kind} / ${state.selected.id}`
    : 'Select a device to begin';
  updateDeviceNav();
  for (const device of state.devices) {
    const btn = document.createElement('button');
    btn.className = `device ${state.selected && state.selected.id === device.id ? 'active' : ''}`;
    btn.innerHTML = `<strong>${device.name}</strong><span class="muted">${device.kind} / ${device.state}</span><span class="muted">${device.id}</span>`;
    btn.onclick = () => {
      state.selected = device;
      renderDevices();
      restartPreview();
    };
    el('devices').appendChild(btn);
  }
  if (!state.devices.length) el('devices').innerHTML = '<p class="muted">No running devices found</p>';
}
function updateDeviceNav() {
  const selectedText = el('selectedMeta').textContent || '';
  const kind = (state.selected && state.selected.kind ? state.selected.kind : selectedText).toLowerCase();
  const platform = kind.includes('ios') ? 'ios' : kind.includes('android') ? 'android' : 'none';
  const nav = el('deviceNav');
  nav.dataset.platform = platform;
  nav.style.gridTemplateColumns = platform === 'ios' ? '1fr' : 'repeat(3, 1fr)';
  const iosOnlyHome = platform === 'ios';
  for (const id of ['navBack', 'navRecents']) {
    const button = el(id);
    button.hidden = iosOnlyHome;
    button.setAttribute('aria-hidden', iosOnlyHome ? 'true' : 'false');
    button.classList.toggle('nav-hidden', iosOnlyHome);
  }
}
setInterval(updateDeviceNav, 300);
async function loadDevices() {
  state.devices = await json('/api/devices');
  if (state.selected && !state.devices.find(d => d.id === state.selected.id)) state.selected = null;
  renderDevices();
  setStatus(`Loaded ${state.devices.length} device(s)`);
}
async function refreshScreenshot() {
  if (!state.selected) return;
  const seq = ++state.previewSeq;
  const controller = new AbortController();
  if (state.pollAbort) state.pollAbort.abort();
  state.pollAbort = controller;
  const res = await api(`/api/devices/${selectedId()}/screenshot`, { signal: controller.signal });
  const blob = await res.blob();
  const url = URL.createObjectURL(blob);
  try {
    await preloadImage(url);
    if (seq !== state.previewSeq) {
      URL.revokeObjectURL(url);
      return;
    }
    showPreviewUrl(url);
  } finally {
    if (state.pollAbort === controller) state.pollAbort = null;
  }
}
function startPolling() {
  stopPreview();
  if (!state.selected) return;
  let active = true;
  const run = async () => {
    if (!active || !state.selected || el('viewMode').value !== 'poll') return;
    const started = performance.now();
    try {
      await refreshScreenshot();
    } catch (err) {
      if (err.name !== 'AbortError') setStatus(err.message);
    }
    const fps = Math.max(1, Number(el('pollFps').value) || 4);
    const delay = Math.max(0, (1000 / fps) - (performance.now() - started));
    state.poll = setTimeout(run, delay);
  };
  state.poll = setTimeout(run, 0);
  state.stopPolling = () => { active = false; };
  setStatus(`Polling ${el('pollFps').value} fps`);
}
function startScreenshotStream() {
  stopPreview();
  if (!state.selected) return;
  const seq = ++state.previewSeq;
  const controller = new AbortController();
  state.streamAbort = controller;
  const params = new URLSearchParams({
    fps: el('streamFps').value,
    format: el('streamFormat').value,
    max_width: el('streamScale').value,
    quality: el('streamQuality').value,
    t: Date.now().toString()
  });
  readScreenshotStream(`/api/devices/${selectedId()}/screenshot-stream?${params}`, controller.signal, seq)
    .catch(err => {
      if (err.name !== 'AbortError') setStatus(err.message);
    });
  setStatus(`Streaming ${el('streamFps').value} fps / ${el('streamFormat').value.toUpperCase()}`);
}
async function startWebRtcStream() {
  stopPreview();
  if (!state.selected) return;
  if (!window.RTCPeerConnection) {
    setStatus('WebRTC unavailable, using HTTP stream');
    startScreenshotStream();
    return;
  }

  const seq = ++state.previewSeq;
  const peer = new RTCPeerConnection({
    iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
  });
  const channel = peer.createDataChannel('appmo-preview', {
    ordered: false,
    maxRetransmits: 0
  });
  channel.binaryType = 'arraybuffer';
  state.webrtcPeer = peer;
  state.webrtcChannel = channel;
  state.webrtcFrames.clear();

  channel.onopen = () => setStatus(`WebRTC ${el('streamFps').value} fps / ${el('streamFormat').value.toUpperCase()}`);
  channel.onclose = () => {
    if (seq === state.previewSeq && el('viewMode').value === 'webrtc') setStatus('WebRTC stream closed');
  };
  channel.onerror = () => {
    if (seq === state.previewSeq) setStatus('WebRTC stream error');
  };
  channel.onmessage = event => {
    handleWebRtcFrame(event.data, seq).catch(err => {
      if (seq === state.previewSeq) setStatus(err.message);
    });
  };
  peer.onconnectionstatechange = () => {
    if (seq !== state.previewSeq) return;
    if (peer.connectionState === 'failed' || peer.connectionState === 'disconnected') {
      setStatus('WebRTC lost, using HTTP stream');
      startScreenshotStream();
    }
  };

  try {
    const offer = await peer.createOffer();
    await peer.setLocalDescription(offer);
    await waitForIceGathering(peer);
    if (seq !== state.previewSeq) return;
    const params = {
      fps: Number(el('streamFps').value),
      format: el('streamFormat').value,
      max_width: Number(el('streamScale').value),
      quality: Number(el('streamQuality').value),
      offer: peer.localDescription
    };
    const response = await json(`/api/devices/${selectedId()}/webrtc/offer`, {
      method: 'POST',
      body: JSON.stringify(params)
    });
    if (seq !== state.previewSeq) return;
    state.webrtcSession = response.session_id;
    await peer.setRemoteDescription(response.answer);
    setStatus('WebRTC connecting');
  } catch (err) {
    if (seq !== state.previewSeq) return;
    stopWebRtc();
    setStatus(`WebRTC unavailable: ${err.message}`);
    startScreenshotStream();
  }
}
function stopPreview() {
  if (state.stopPolling) {
    state.stopPolling();
    state.stopPolling = null;
  }
  clearTimeout(state.poll);
  state.poll = null;
  if (state.pollAbort) {
    state.pollAbort.abort();
    state.pollAbort = null;
  }
  if (state.streamAbort) {
    state.streamAbort.abort();
    state.streamAbort = null;
  }
  stopWebRtc();
  state.previewSeq++;
}
function stopWebRtc() {
  state.webrtcFrames.clear();
  state.webrtcSession = null;
  if (state.webrtcChannel) {
    state.webrtcChannel.onopen = null;
    state.webrtcChannel.onclose = null;
    state.webrtcChannel.onerror = null;
    state.webrtcChannel.onmessage = null;
    try { state.webrtcChannel.close(); } catch (_) {}
    state.webrtcChannel = null;
  }
  if (state.webrtcPeer) {
    state.webrtcPeer.onconnectionstatechange = null;
    try { state.webrtcPeer.close(); } catch (_) {}
    state.webrtcPeer = null;
  }
}
async function readScreenshotStream(path, signal, seq) {
  const res = await fetch(path, { signal, cache: 'no-store' });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || `${res.status} ${res.statusText}`);
  }
  const reader = res.body.getReader();
  let buffer = new Uint8Array(0);
  const delimiter = new TextEncoder().encode('\r\n\r\n');
  while (seq === state.previewSeq) {
    const { value, done } = await reader.read();
    if (done) break;
    buffer = appendBytes(buffer, value);
    while (true) {
      const headerEnd = indexOfBytes(buffer, delimiter);
      if (headerEnd < 0) break;
      const header = new TextDecoder().decode(buffer.slice(0, headerEnd));
      const lengthMatch = header.match(/Content-Length:\s*(\d+)/i);
      if (!lengthMatch) {
        buffer = buffer.slice(headerEnd + delimiter.length);
        continue;
      }
      const length = Number(lengthMatch[1]);
      const frameStart = headerEnd + delimiter.length;
      const frameEnd = frameStart + length;
      if (buffer.length < frameEnd) break;
      const typeMatch = header.match(/Content-Type:\s*([^\r\n]+)/i);
      const frame = buffer.slice(frameStart, frameEnd);
      buffer = buffer.slice(frameEnd);
      const url = URL.createObjectURL(new Blob([frame], { type: typeMatch ? typeMatch[1].trim() : 'image/jpeg' }));
      await preloadImage(url);
      if (seq !== state.previewSeq) {
        URL.revokeObjectURL(url);
        return;
      }
      showPreviewUrl(url);
      await nextAnimationFrame();
    }
  }
}
function waitForIceGathering(peer) {
  if (peer.iceGatheringState === 'complete') return Promise.resolve();
  return new Promise(resolve => {
    const timeout = setTimeout(done, 1200);
    function done() {
      clearTimeout(timeout);
      peer.removeEventListener('icegatheringstatechange', onChange);
      resolve();
    }
    function onChange() {
      if (peer.iceGatheringState === 'complete') done();
    }
    peer.addEventListener('icegatheringstatechange', onChange);
  });
}
async function handleWebRtcFrame(data, seq) {
  if (seq !== state.previewSeq) return;
  const bytes = new Uint8Array(data);
  if (bytes.length < 10) return;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const frameSeq = view.getUint32(0);
  const flags = bytes[4];
  const typeLength = bytes[5];
  const totalLength = view.getUint32(6);
  const payloadStart = 10 + typeLength;
  if (payloadStart > bytes.length) return;

  let frame = state.webrtcFrames.get(frameSeq);
  if (flags & 1) {
    const contentType = new TextDecoder().decode(bytes.slice(10, payloadStart)) || 'image/jpeg';
    frame = { contentType, totalLength, chunks: [], received: 0 };
    state.webrtcFrames.set(frameSeq, frame);
    trimWebRtcFrames();
  }
  if (!frame) return;

  const chunk = bytes.slice(payloadStart);
  frame.chunks.push(chunk);
  frame.received += chunk.length;
  if (!(flags & 2)) return;

  state.webrtcFrames.delete(frameSeq);
  const merged = new Uint8Array(frame.totalLength || frame.received);
  let offset = 0;
  for (const part of frame.chunks) {
    merged.set(part.subarray(0, Math.min(part.length, merged.length - offset)), offset);
    offset += part.length;
    if (offset >= merged.length) break;
  }
  const url = URL.createObjectURL(new Blob([merged], { type: frame.contentType }));
  await preloadImage(url);
  if (seq !== state.previewSeq) {
    URL.revokeObjectURL(url);
    return;
  }
  showPreviewUrl(url);
  await nextAnimationFrame();
}
function trimWebRtcFrames() {
  while (state.webrtcFrames.size > 8) {
    const oldest = state.webrtcFrames.keys().next().value;
    state.webrtcFrames.delete(oldest);
  }
}
function appendBytes(left, right) {
  const merged = new Uint8Array(left.length + right.length);
  merged.set(left);
  merged.set(right, left.length);
  return merged;
}
function indexOfBytes(buffer, needle) {
  outer:
  for (let i = 0; i <= buffer.length - needle.length; i++) {
    for (let j = 0; j < needle.length; j++) {
      if (buffer[i + j] !== needle[j]) continue outer;
    }
    return i;
  }
  return -1;
}
function nextAnimationFrame() {
  return new Promise(resolve => requestAnimationFrame(resolve));
}
function preloadImage(url) {
  const image = new Image();
  image.decoding = 'async';
  image.src = url;
  if (image.decode) return image.decode();
  return new Promise((resolve, reject) => {
    image.onload = resolve;
    image.onerror = reject;
  });
}
function showPreviewUrl(url) {
  const screen = el('screen');
  const previousUrl = state.previewUrl;
  state.previewUrl = url;
  screen.src = url;
  screen.style.display = 'block';
  el('screenEmpty').style.display = 'none';
  if (previousUrl) requestAnimationFrame(() => URL.revokeObjectURL(previousUrl));
}
function restartPreview() {
  updateDeviceNav();
  if (el('viewMode').value === 'webrtc') {
    startWebRtcStream();
  } else if (el('viewMode').value === 'stream') {
    startScreenshotStream();
  } else {
    startPolling();
  }
}
function openSettings() {
  const modal = el('settingsModal');
  modal.classList.add('open');
  modal.setAttribute('aria-hidden', 'false');
  el('viewMode').focus();
}
function closeSettings() {
  const modal = el('settingsModal');
  modal.classList.remove('open');
  modal.setAttribute('aria-hidden', 'true');
  el('settingsOpen').focus();
}
function isPreviewFullscreen() {
  const activeElement = document.fullscreenElement || document.webkitFullscreenElement;
  return activeElement === el('screenWrap');
}
function updateFullscreenButton() {
  const button = el('fullscreenToggle');
  const active = isPreviewFullscreen();
  button.title = active ? 'Exit fullscreen' : 'Enter fullscreen';
  button.setAttribute('aria-label', button.title);
  button.classList.toggle('active', active);
}
async function toggleFullscreen() {
  const preview = el('screenWrap');
  const requestFullscreen = preview && (preview.requestFullscreen || preview.webkitRequestFullscreen);
  const exitFullscreen = document.exitFullscreen || document.webkitExitFullscreen;
  const fullscreenEnabled = document.fullscreenEnabled || document.webkitFullscreenEnabled;
  if (!fullscreenEnabled || !preview || !requestFullscreen || !exitFullscreen) {
    setStatus('Fullscreen is not supported');
    return;
  }
  if (isPreviewFullscreen()) {
    await exitFullscreen.call(document);
  } else {
    await requestFullscreen.call(preview);
  }
  updateFullscreenButton();
}
async function post(path, body) {
  await json(path, { method: 'POST', body: JSON.stringify(body || {}) });
  setStatus('Command sent');
}
function wsReady() {
  return state.ws && state.ws.readyState === WebSocket.OPEN;
}
function sendWsControl(type, payload) {
  if (!state.selected || !wsReady()) return null;
  const requestId = `${Date.now()}-${++state.requestSeq}`;
  const message = { request_id: requestId, device_id: state.selected.id, type, ...payload };
  const promise = new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      state.pending.delete(requestId);
      reject(new Error('Control command timed out'));
    }, 2000);
    state.pending.set(requestId, { resolve, reject, timeout });
  });
  state.ws.send(JSON.stringify(message));
  return promise;
}
async function control(type, payload, restPath, restBody = payload) {
  const wsPromise = sendWsControl(type, payload);
  if (wsPromise) {
    await wsPromise;
    setStatus('Command sent');
    return;
  }
  await post(restPath, restBody);
}
function imagePoint(event) {
  return clientPointToImage(event.clientX, event.clientY);
}
function pagePointToImage(pageX, pageY) {
  return clientPointToImage(pageX - window.scrollX, pageY - window.scrollY);
}
function clientPointToImage(clientX, clientY) {
  const img = el('screen');
  if (!img.naturalWidth || !img.naturalHeight) throw new Error('Screenshot is not ready');
  const rect = img.getBoundingClientRect();
  const x = Math.max(0, Math.min(rect.width, clientX - rect.left));
  const y = Math.max(0, Math.min(rect.height, clientY - rect.top));
  return {
    x: Math.round(x * img.naturalWidth / rect.width),
    y: Math.round(y * img.naturalHeight / rect.height),
    source_width: img.naturalWidth,
    source_height: img.naturalHeight
  };
}
async function sendPointerCommand(start, end) {
  const dx = Math.abs(end.x - start.x);
  const dy = Math.abs(end.y - start.y);
  if (dx < 8 && dy < 8) {
    await control('tap', end, `/api/devices/${selectedId()}/input/tap`);
    setStatus(`Tapped ${end.x}, ${end.y}`);
  } else {
    const payload = {
      x1: start.x,
      y1: start.y,
      x2: end.x,
      y2: end.y,
      duration_ms: 250,
      source_width: start.source_width,
      source_height: start.source_height
    };
    await control('swipe', payload, `/api/devices/${selectedId()}/input/swipe`);
    setStatus(`Swiped ${start.x}, ${start.y} -> ${end.x}, ${end.y}`);
  }
  if (el('viewMode').value === 'poll') {
    refreshScreenshot().catch(err => setStatus(err.message));
  }
}
async function sendKeyValue(key) {
  await control('key', { key }, `/api/devices/${selectedId()}/key`);
  setStatus(`Sent ${key}`);
}
async function loadLogs() {
  const res = await api(`/api/devices/${selectedId()}/logs?lines=300`);
  el('logs').textContent = await res.text();
}
function connectWs() {
  if (state.ws) state.ws.close();
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  state.ws = new WebSocket(`${proto}://${location.host}/ws`);
  state.ws.onmessage = ev => {
    try {
      const msg = JSON.parse(ev.data);
      if (msg.type === 'control_result' && msg.request_id) {
        const pending = state.pending.get(msg.request_id);
        if (!pending) return;
        clearTimeout(pending.timeout);
        state.pending.delete(msg.request_id);
        msg.ok ? pending.resolve(msg) : pending.reject(new Error(msg.error || 'Control command failed'));
        return;
      }
    } catch (_) {
      // Plain status messages are still supported for compatibility.
    }
    setStatus(ev.data);
  };
  state.ws.onopen = () => setStatus('Connected');
  state.ws.onclose = () => {
    for (const pending of state.pending.values()) {
      clearTimeout(pending.timeout);
      pending.reject(new Error('WebSocket disconnected'));
    }
    state.pending.clear();
    setStatus('WebSocket disconnected');
    setTimeout(connectWs, 1000);
  };
}
el('refresh').onclick = () => loadDevices().catch(err => setStatus(err.message));
el('fullscreenToggle').onclick = () => toggleFullscreen().catch(err => setStatus(err.message));
document.addEventListener('fullscreenchange', updateFullscreenButton);
document.addEventListener('webkitfullscreenchange', updateFullscreenButton);
el('settingsOpen').onclick = () => openSettings();
el('settingsClose').onclick = () => closeSettings();
el('settingsModal').onclick = event => {
  if (event.target === el('settingsModal')) closeSettings();
};
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && el('settingsModal').classList.contains('open')) closeSettings();
});
el('shot').onclick = () => refreshScreenshot().catch(err => setStatus(err.message));
el('viewMode').onchange = () => restartPreview();
el('pollFps').onchange = () => { if (el('viewMode').value === 'poll') startPolling(); };
el('streamFps').onchange = () => { if (el('viewMode').value !== 'poll') restartPreview(); };
el('streamFormat').onchange = () => { if (el('viewMode').value !== 'poll') restartPreview(); };
el('streamScale').onchange = () => { if (el('viewMode').value !== 'poll') restartPreview(); };
el('streamQuality').onchange = () => { if (el('viewMode').value !== 'poll') restartPreview(); };
el('logsBtn').onclick = () => loadLogs().catch(err => setStatus(err.message));
el('navBack').onclick = () => sendKeyValue('BACK').catch(err => setStatus(err.message));
el('navHome').onclick = () => sendKeyValue('HOME').catch(err => setStatus(err.message));
el('navRecents').onclick = () => sendKeyValue('APP_SWITCH').catch(err => setStatus(err.message));
el('install').onclick = () => post(`/api/devices/${selectedId()}/app/install`, { path: el('appPath').value }).catch(err => setStatus(err.message));
el('launch').onclick = () => post(`/api/devices/${selectedId()}/app/launch`, { app_id: el('appId').value }).catch(err => setStatus(err.message));
el('terminate').onclick = () => post(`/api/devices/${selectedId()}/app/terminate`, { app_id: el('appId').value }).catch(err => setStatus(err.message));
el('recordStart').onclick = () => post(`/api/devices/${selectedId()}/record/start`, {}).catch(err => setStatus(err.message));
el('recordStop').onclick = () => post(`/api/devices/${selectedId()}/record/stop`, {}).catch(err => setStatus(err.message));
function setupScreenControls() {
  if (window.interact) {
    document.body.dataset.touchBackend = 'interact.js';
    interact('#screen')
      .draggable({
        listeners: {
          end(event) {
            try {
              if (!state.selected) throw new Error('Select a device first');
              const start = pagePointToImage(event.x0, event.y0);
              const end = pagePointToImage(event.pageX, event.pageY);
              if (Math.abs(end.x - start.x) >= 8 || Math.abs(end.y - start.y) >= 8) {
                sendPointerCommand(start, end).catch(err => setStatus(err.message));
              }
            } catch (err) {
              setStatus(err.message);
            }
          }
        }
      })
      .on('tap', event => {
        try {
          if (!state.selected) throw new Error('Select a device first');
          const point = imagePoint(event);
          event.preventDefault();
          sendPointerCommand(point, point).catch(err => setStatus(err.message));
        } catch (err) {
          setStatus(err.message);
        }
      });
    return;
  }

  document.body.dataset.touchBackend = 'pointer-events-fallback';
  el('screen').addEventListener('pointerdown', event => {
    try {
      if (!state.selected) throw new Error('Select a device first');
      el('screen').setPointerCapture(event.pointerId);
      state.pointerStart = imagePoint(event);
      event.preventDefault();
    } catch (err) {
      setStatus(err.message);
    }
  });
  el('screen').addEventListener('pointerup', event => {
    try {
      if (!state.pointerStart) return;
      const start = state.pointerStart;
      state.pointerStart = null;
      event.preventDefault();
      sendPointerCommand(start, imagePoint(event)).catch(err => setStatus(err.message));
    } catch (err) {
      state.pointerStart = null;
      setStatus(err.message);
    }
  });
  el('screen').addEventListener('pointercancel', () => { state.pointerStart = null; });
}
setupScreenControls();
connectWs();
loadDevices().catch(err => setStatus(err.message));
"#;
