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
  <title>Appmon</title>
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
                            h1 { class: "brand-title", "Appmon" }
                        }
                    }
                    DevicesPane {}
                }
                MonitorPane {}
                AppControls {}
                div { id: "sidebarBackdrop", class: "sidebar-backdrop", aria_hidden: "true" }
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
                    div { class: "monitor-heading",
                        button { id: "sidebarToggle", class: "icon-btn mobile-sidebar-toggle", title: "Open devices", aria_label: "Open devices", aria_expanded: "false", aria_controls: "devices",
                            span { class: "hamburger-icon" }
                        }
                        div {
                            h2 { class: "monitor-title", "Monitor" }
                            span { id: "selectedMeta", class: "text-xs text-slate-500", "No device selected" }
                        }
                    }
                    div { class: "toolbar-actions",
                        span { id: "statusChip", class: "status-pill", "Idle" }
                        button { id: "fullscreenToggle", class: "icon-btn", title: "Enter fullscreen", aria_label: "Enter fullscreen",
                            span { class: "fullscreen-icon" }
                        }
                        button { id: "appOpen", class: "btn btn-secondary", title: "Open app controls", aria_label: "Open app controls", "App" }
                        button { id: "settingsOpen", class: "btn btn-secondary", title: "Preview controls", aria_label: "Preview controls", "View" }
                    }
                }
                div { id: "screenWrap", class: "screen-wrap",
                    canvas { id: "screenCanvas", aria_label: "Device screenshot preview" }
                    img { id: "screenStream", alt: "Device stream preview" }
                    video { id: "screenVideo", autoplay: true, muted: true, playsinline: true, controls: false, aria_label: "Device video preview" }
                    div { id: "screenEmpty", class: "empty-screen", "Select a device" }
                    button { id: "fullscreenExit", class: "fullscreen-exit", title: "Exit fullscreen", aria_label: "Exit fullscreen",
                        span { class: "close-icon" }
                    }
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
                    }
                    button { id: "settingsClose", class: "icon-btn", title: "Close", aria_label: "Close",
                        span { class: "close-icon" }
                    }
                }
                div { class: "settings-grid",
                    label { id: "viewModeField", class: "settings-field",
                        span { "Mode" }
                        select { id: "viewMode", title: "Preview mode", aria_label: "Preview mode",
                            option { value: "webrtc", selected: true, "WebRTC" }
                            option { value: "poll", "Polling" }
                            option { value: "stream", "Stream" }
                        }
                    }
                    label { id: "pollFpsField", class: "settings-field",
                        span { "Polling FPS" }
                        select { id: "pollFps", title: "Polling FPS", aria_label: "Polling FPS",
                            option { value: "1", "1 fps" }
                            option { value: "2", "2 fps" }
                            option { value: "4", selected: true, "4 fps" }
                            option { value: "6", "6 fps" }
                            option { value: "8", "8 fps" }
                        }
                    }
                    label { id: "streamFpsField", class: "settings-field",
                        span { "Video FPS" }
                        select { id: "streamFps", title: "WebRTC and fallback stream FPS", aria_label: "WebRTC and fallback stream FPS",
                            option { value: "4", "4 fps" }
                            option { value: "8", "8 fps" }
                            option { value: "12", "12 fps" }
                            option { value: "15", selected: true, "15 fps" }
                            option { value: "20", "20 fps" }
                        }
                    }
                    label { id: "streamFormatField", class: "settings-field",
                        span { "Transport" }
                        select { id: "streamFormat", title: "WebRTC transport preference", aria_label: "WebRTC transport preference",
                            option { value: "auto", selected: true, "Appmon WebRTC" }
                            option { value: "video", "Fast video" }
                            option { value: "native", "Native emulator" }
                            option { value: "jpeg", "Small JPEG" }
                        }
                    }
                    label { id: "streamScaleField", class: "settings-field",
                        span { "Scale" }
                        select { id: "streamScale", title: "Fallback stream scale", aria_label: "Fallback stream scale",
                            option { value: "540", "540p" }
                            option { value: "720", selected: true, "720p" }
                            option { value: "1080", "1080p" }
                            option { value: "4096", "Full" }
                        }
                    }
                    label { id: "streamQualityField", class: "settings-field",
                        span { "Quality" }
                        select { id: "streamQuality", title: "Fallback stream quality", aria_label: "Fallback stream quality",
                            option { value: "55", "Eco" }
                            option { value: "70", selected: true, "Balanced" }
                            option { value: "85", "Sharp" }
                        }
                    }
                }
                div { id: "settingsFeedback", class: "settings-feedback", role: "status", aria_live: "polite", "Ready" }
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
        div { id: "logsPanel", class: "logs-panel",
            div { class: "section-head compact",
                div {
                    h2 { class: "section-title", "Logs" }
                }
            }
            pre { id: "logs" }
        }
    }
}

#[component]
fn AppControls() -> Element {
    rsx! {
        aside { id: "appPanel", class: "app-controls panel-section", aria_hidden: "true",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "App" }
                }
                button { id: "appClose", class: "icon-btn", title: "Close app controls", aria_label: "Close app controls",
                    span { class: "close-icon" }
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
.hamburger-icon {
  width: 18px;
  height: 14px;
  position: relative;
  display: block;
  border-top: 2px solid currentColor;
  border-bottom: 2px solid currentColor;
}
.hamburger-icon::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  top: 4px;
  height: 2px;
  border-radius: 999px;
  background: currentColor;
}
.mobile-sidebar-toggle,
.sidebar-backdrop {
  display: none;
}
.screen-wrap:fullscreen,
.screen-wrap:-webkit-full-screen,
.screen-wrap.fallback-fullscreen {
  width: 100vw;
  height: 100vh;
  height: 100dvh;
  border: 0;
  border-radius: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #090d14;
}
.screen-wrap.fallback-fullscreen {
  position: fixed;
  inset: 0;
  z-index: 1000;
}
.fullscreen-exit {
  position: absolute;
  top: max(14px, env(safe-area-inset-top));
  right: max(14px, env(safe-area-inset-right));
  z-index: 1001;
  width: 42px;
  height: 42px;
  display: none;
  place-items: center;
  border: 1px solid rgba(255, 255, 255, .18);
  border-radius: var(--theme-radius);
  background: rgba(15, 23, 42, .72);
  color: var(--tw-white);
  box-shadow: 0 14px 34px rgba(0, 0, 0, .28);
}
.screen-wrap:fullscreen .fullscreen-exit,
.screen-wrap:-webkit-full-screen .fullscreen-exit,
.screen-wrap.fallback-fullscreen .fullscreen-exit {
  display: grid;
}
.screen-wrap:fullscreen #screenCanvas,
.screen-wrap:fullscreen #screenStream,
.screen-wrap:fullscreen #screenVideo,
.screen-wrap:-webkit-full-screen #screenCanvas,
.screen-wrap:-webkit-full-screen #screenStream,
.screen-wrap:-webkit-full-screen #screenVideo,
.screen-wrap.fallback-fullscreen #screenCanvas,
.screen-wrap.fallback-fullscreen #screenStream,
.screen-wrap.fallback-fullscreen #screenVideo {
  max-width: 100%;
  max-height: 100%;
}
body.preview-fullscreen-lock {
  overflow: hidden;
  touch-action: none;
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
.monitor-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
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
.settings-field {
  display: grid;
  gap: 5px;
}
.settings-field span {
  color: var(--ink-muted);
  font-size: 11px;
  font-weight: 760;
  line-height: 1.2;
}
.settings-grid select,
.settings-actions .btn {
  width: 100%;
}
.settings-feedback {
  min-height: 38px;
  margin-top: 12px;
  border: 1px solid var(--theme-soft-line);
  border-radius: var(--theme-radius);
  padding: 9px 11px;
  background: #f8fafc;
  color: var(--tw-slate-700);
  font-size: 12px;
  font-weight: 720;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  transition: background .16s ease, border-color .16s ease, color .16s ease;
}
.settings-feedback.success {
  border-color: rgba(15, 118, 110, .28);
  background: #ecfdf5;
  color: #0f766e;
}
.settings-feedback.error {
  border-color: rgba(220, 38, 38, .28);
  background: #fef2f2;
  color: #b91c1c;
}
.settings-feedback.working {
  border-color: rgba(51, 65, 85, .24);
  background: #f1f5f9;
  color: #334155;
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
.btn.is-working,
.btn.is-done {
  position: relative;
}
.btn.is-working {
  opacity: .76;
  pointer-events: none;
}
.btn.is-done {
  border-color: rgba(15, 118, 110, .4);
  background: #0f766e;
  color: #fff;
}
.btn-secondary.is-done {
  background: #ecfdf5;
  color: #0f766e;
}
.btn-danger.is-done {
  background: #991b1b;
  border-color: #991b1b;
}
.btn.is-working::after {
  content: "";
  width: 13px;
  height: 13px;
  margin-left: 8px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 999px;
  display: inline-block;
  vertical-align: -2px;
  animation: spin .7s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
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
.device-group {
  display: grid;
  gap: 8px;
}
.device-group + .device-group {
  margin-top: 12px;
}
.device-group-title {
  margin: 0;
  color: var(--tw-slate-500);
  font-size: 11px;
  font-weight: 820;
  text-transform: uppercase;
}
.settings-grid select {
  width: 100%;
  min-height: 40px;
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  background: var(--tw-white);
  color: var(--tw-slate-900);
  padding: 0 11px;
  font: inherit;
  box-shadow: var(--shadow-soft);
}
.device {
  width: 100%;
  min-height: 82px;
  text-align: left;
  cursor: pointer;
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
.device-main {
  min-width: 0;
  display: grid;
  gap: 5px;
}
.device-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}
.device-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}
.device-action {
  min-height: 30px;
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  background: var(--tw-white);
  color: var(--tw-slate-700);
  padding: 5px 9px;
  font-size: 12px;
  font-weight: 760;
}
.device-action.stop {
  border-color: rgba(217, 75, 63, .34);
  color: var(--tw-coral-600);
}
.device-action:disabled {
  cursor: wait;
  opacity: .62;
}
.device strong { font-size: 15px; }
.device.active {
  border-color: rgba(21, 166, 163, .58);
  outline: 3px solid var(--theme-ring);
  background: #effdfb;
}
.device:focus-visible {
  outline: 3px solid var(--theme-ring);
  outline-offset: 1px;
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
#screenCanvas,
#screenStream,
#screenVideo {
  max-width: 100%;
  max-height: min(66vh, 620px);
  object-fit: contain;
  display: none;
  cursor: crosshair;
  user-select: none;
  image-rendering: auto;
  transform: translateZ(0);
  will-change: contents;
}
#screenVideo {
  background: #090d14;
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
.device-nav[data-platform="none"] {
  display: none;
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
  #screenCanvas,
  #screenStream,
  #screenVideo { max-height: 58vh; }
  .action-grid { grid-template-columns: 1fr; }
  .flex-wrap, .grid-cols-3 { grid-template-columns: 1fr; display: grid; }
}

/* Minimal modern dashboard skin. Kept as CSS overrides so existing device
   preview, touch input, and fullscreen JavaScript ids remain stable. */
:root {
  --theme-line: #e2e8f0;
  --theme-soft-line: #edf2f7;
  --theme-ring: rgba(15, 23, 42, .12);
  --surface: #ffffff;
  --surface-strong: #ffffff;
  --surface-muted: #f8fafc;
  --ink: #0f172a;
  --ink-muted: #64748b;
  --brand-a: #0f172a;
  --brand-b: #334155;
  --brand-c: #0f766e;
  --theme-radius: 8px;
  --shadow-panel: 0 1px 2px rgba(15, 23, 42, .04);
  --shadow-soft: 0 1px 2px rgba(15, 23, 42, .03);
}
body {
  background: #f8fafc;
  color: var(--ink);
}
input, textarea, select {
  min-height: 40px;
  border-color: var(--theme-line);
  background: #ffffff;
  box-shadow: none;
}
input::placeholder { color: #94a3b8; }
select {
  appearance: none;
  background-image:
    linear-gradient(45deg, transparent 50%, #64748b 50%),
    linear-gradient(135deg, #64748b 50%, transparent 50%);
  background-position:
    calc(100% - 17px) calc(50% - 3px),
    calc(100% - 12px) calc(50% - 3px);
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
  padding-right: 34px;
}
.app-shell {
  background: #f8fafc;
}
.app-layout {
  grid-template-columns: 300px minmax(420px, 1fr) 320px;
  gap: 12px;
  padding: 12px;
}
.workspace-rail,
.monitor-pane,
.app-controls,
.monitor-card,
.logs-panel,
.settings-dialog {
  border-color: var(--theme-soft-line);
  background: var(--surface);
  box-shadow: var(--shadow-panel);
  backdrop-filter: none;
}
.workspace-rail,
.monitor-pane,
.app-controls {
  min-height: calc(100vh - 24px);
}
.monitor-pane {
  padding: 0;
  background: transparent;
  box-shadow: none;
  border: 0;
}
.brand-lockup {
  padding: 16px;
  background: #ffffff;
  color: var(--ink);
  border-bottom: 1px solid var(--theme-soft-line);
}
.brand-mark {
  width: 36px;
  height: 36px;
  background: #0f172a;
  box-shadow: none;
}
.brand-title {
  font-size: 17px;
  letter-spacing: 0;
}
.brand-subtitle {
  color: var(--ink-muted);
}
.panel-section {
  padding: 14px;
}
.section-head {
  margin-bottom: 12px;
}
.section-title {
  font-size: 14px;
  letter-spacing: 0;
}
.section-kicker,
.muted,
.brand-subtitle {
  color: var(--ink-muted);
}
.brand-subtitle { color: var(--ink-muted); }
.monitor-card {
  padding: 12px;
  background: var(--surface-strong);
}
.monitor-toolbar {
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--theme-soft-line);
}
.monitor-title {
  font-size: 20px;
  letter-spacing: 0;
}
#selectedMeta {
  display: inline-block;
  max-width: min(52vw, 620px);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.icon-btn,
.btn {
  min-height: 40px;
  border-radius: var(--theme-radius);
}
.icon-btn {
  background: #ffffff;
  box-shadow: none;
}
.btn {
  background: #0f172a;
  box-shadow: none;
}
.btn-secondary {
  background: #ffffff;
  border-color: var(--theme-line);
  color: var(--ink);
}
.btn-danger {
  background: #dc2626;
  border-color: #dc2626;
}
.btn:hover,
.icon-btn:hover,
.device:hover {
  transform: none;
  box-shadow: none;
  border-color: #cbd5e1;
}
.status-pill {
  border-color: var(--theme-line);
  background: #f8fafc;
  color: #334155;
  border-radius: 999px;
}
.settings-grid select {
  background-color: #fff;
  box-shadow: none;
}
.device-list {
  gap: 8px;
}
.device-group-title {
  color: #475569;
  letter-spacing: 0;
}
.device {
  min-height: 78px;
  border-color: var(--theme-soft-line);
  background: #fff;
  box-shadow: none;
}
.device-action {
  background: #ffffff;
  border-color: var(--theme-line);
  color: #334155;
  box-shadow: none;
}
.device-action.stop {
  border-color: #fecaca;
  color: #b91c1c;
}
.device-action:hover {
  border-color: #cbd5e1;
  background: #f8fafc;
}
.device strong {
  color: var(--ink);
  font-size: 14px;
}
.device.active {
  border-color: #0f172a;
  background: #f8fafc;
  outline: 2px solid var(--theme-ring);
}
.screen-wrap {
  min-height: min(70vh, 680px);
  border-color: rgba(15, 23, 42, .92);
  border-radius: var(--theme-radius) var(--theme-radius) 0 0;
  background: #020617;
}
#screenCanvas,
#screenStream,
#screenVideo {
  max-height: min(70vh, 680px);
}
.empty-screen {
  color: rgba(226, 232, 240, .84);
  font-size: 14px;
}
.device-nav {
  height: 44px;
  background: #0f172a;
}
.device-nav button:hover {
  background: rgba(255, 255, 255, .08);
  color: #fff;
}
.logs-panel {
  margin-top: 12px;
  padding: 12px;
  background: var(--surface-strong);
}
pre {
  max-height: 300px;
  border-color: rgba(15, 23, 42, .86);
  background: #020617;
  box-shadow: none;
}
.field-band {
  padding: 12px;
  background: var(--surface-muted);
}
.action-grid {
  gap: 9px;
}
.settings-modal {
  background: rgba(15, 23, 42, .32);
}
.settings-dialog {
  background: #ffffff;
}
@media (max-width: 1240px) {
  .app-layout { grid-template-columns: 288px minmax(0, 1fr); }
  .app-controls { min-height: auto; }
}
@media (max-width: 920px) {
  .app-layout {
    grid-template-columns: minmax(0, 1fr);
    padding: 12px;
  }
  .mobile-sidebar-toggle {
    display: grid;
    flex: 0 0 auto;
  }
  .workspace-rail {
    position: fixed;
    inset: 0 auto 0 0;
    z-index: 70;
    width: min(84vw, 320px);
    height: 100vh;
    height: 100dvh;
    min-height: 100vh;
    min-height: 100dvh;
    max-height: 100vh;
    max-height: 100dvh;
    border-radius: 0 var(--theme-radius) var(--theme-radius) 0;
    border-left: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transform: translateX(calc(-100% - 16px));
    transition: transform .18s ease;
  }
  .workspace-rail .brand-lockup {
    flex: 0 0 auto;
  }
  .workspace-rail .devices-pane {
    flex: 1 1 auto;
    min-height: 0;
    height: auto;
    max-height: none;
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
  }
  body.sidebar-open .workspace-rail {
    transform: translateX(0);
  }
  .sidebar-backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: block;
    pointer-events: none;
    background: rgba(15, 23, 42, .32);
    opacity: 0;
    transition: opacity .18s ease;
  }
  body.sidebar-open .sidebar-backdrop {
    pointer-events: auto;
    opacity: 1;
  }
  body.sidebar-lock {
    overflow: hidden;
  }
  .workspace-rail,
  .monitor-pane,
  .app-controls {
    min-height: auto;
  }
  .workspace-rail {
    min-height: 100vh;
    min-height: 100dvh;
  }
  .monitor-pane { order: -1; }
  #selectedMeta {
    max-width: calc(100vw - 64px);
    white-space: normal;
  }
  .screen-wrap { min-height: 62vh; }
  #screenCanvas,
  #screenStream,
  #screenVideo { max-height: 62vh; }
}
@media (max-width: 560px) {
  .app-shell { background: #f8fafc; }
  .app-layout { padding: 0; gap: 0; }
  .monitor-card,
  .logs-panel,
  .workspace-rail,
  .app-controls {
    border-radius: 0;
  }
  .monitor-card {
    border-left: 0;
    border-right: 0;
  }
  .monitor-title { font-size: 21px; }
  .monitor-toolbar {
    grid-template-columns: minmax(0, 1fr);
    width: 100%;
  }
  .toolbar-actions {
    grid-template-columns: 42px minmax(0, 1fr);
    justify-self: stretch;
    width: 100%;
  }
  .toolbar-actions .status-pill { grid-column: 1 / -1; }
  .toolbar-actions .icon-btn { width: 42px; }
  .toolbar-actions .btn { width: 100%; }
  .screen-wrap { min-height: 58vh; }
  #screenCanvas,
  #screenStream,
  #screenVideo { max-height: 58vh; }
}

/* Dark control-room skin. This final layer intentionally owns the visible
   palette while preserving the existing DOM and device interaction ids. */
:root {
  color-scheme: dark;
  --theme-line: rgba(111, 107, 255, .28);
  --theme-soft-line: rgba(255, 255, 255, .075);
  --theme-ring: rgba(70, 66, 255, .34);
  --surface: rgba(22, 21, 32, .94);
  --surface-strong: #1c1b2b;
  --surface-muted: rgba(39, 38, 58, .82);
  --surface-control: rgba(13, 13, 22, .86);
  --ink: #f7f7ff;
  --ink-muted: #aaa8c8;
  --ink-soft: #d9d8f5;
  --brand-a: #4642ff;
  --brand-b: #736dff;
  --brand-c: #9d8cff;
  --shadow-panel: 0 26px 80px rgba(0, 0, 0, .42);
  --shadow-soft: 0 14px 34px rgba(0, 0, 0, .26);
}
body {
  background: #08070d;
  color: var(--ink);
}
input,
textarea,
select {
  border-color: var(--theme-line);
  background-color: var(--surface-control);
  color: var(--ink);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, .035);
}
input::placeholder { color: #73718f; }
select {
  background-image:
    linear-gradient(45deg, transparent 50%, #736dff 50%),
    linear-gradient(135deg, #736dff 50%, transparent 50%);
}
pre {
  border-color: rgba(111, 107, 255, .18);
  background: #08070d;
  color: #e9e8ff;
}
.text-slate-950,
.text-slate-700,
.section-title,
.brand-title,
.monitor-title {
  color: var(--ink);
}
.text-slate-500,
.section-kicker,
.brand-subtitle,
.muted,
#selectedMeta,
#status {
  color: var(--ink-muted);
}
.app-shell {
  background:
    radial-gradient(circle at 64% 10%, rgba(70, 66, 255, .16), transparent 34%),
    linear-gradient(135deg, rgba(115, 109, 255, .1), transparent 340px),
    #08070d;
}
.app-layout {
  gap: 14px;
  padding: 14px;
}
.workspace-rail,
.app-controls,
.monitor-card,
.logs-panel,
.settings-dialog {
  border-color: var(--theme-soft-line);
  background: var(--surface);
  box-shadow: var(--shadow-panel);
}
.workspace-rail,
.app-controls,
.monitor-card,
.logs-panel {
  backdrop-filter: blur(18px);
}
.monitor-pane {
  background: transparent;
}
.brand-lockup {
  background: linear-gradient(180deg, rgba(31, 30, 45, .94), rgba(18, 17, 28, .82));
  border-bottom-color: var(--theme-soft-line);
}
.brand-mark {
  color: #fff;
  background: linear-gradient(135deg, #4642ff, #817bff);
  box-shadow: 0 14px 34px rgba(70, 66, 255, .24);
}
.section-head {
  border-color: var(--theme-soft-line);
}
.icon-btn,
.btn,
.device-action {
  border-color: var(--theme-line);
  color: var(--ink);
  background: linear-gradient(180deg, rgba(30, 41, 59, .88), rgba(15, 23, 42, .9));
  box-shadow: 0 10px 24px rgba(0, 0, 0, .2);
}
.icon-btn {
  color: var(--ink-soft);
}
.icon-btn:hover,
.btn:hover,
.device-action:hover {
  border-color: rgba(115, 109, 255, .58);
  background: linear-gradient(180deg, rgba(70, 66, 255, .24), rgba(16, 15, 28, .94));
  color: #f8ffff;
  transform: translateY(-1px);
}
.icon-btn.active {
  border-color: rgba(115, 109, 255, .68);
  background: rgba(70, 66, 255, .18);
  color: #b9b5ff;
}
.btn {
  border-color: rgba(115, 109, 255, .5);
  background: linear-gradient(180deg, #4f46ff, #332dff);
  color: #fff;
}
.btn-secondary {
  border-color: var(--theme-line);
  background: linear-gradient(180deg, rgba(30, 41, 59, .92), rgba(15, 23, 42, .88));
  color: var(--ink);
}
.btn-danger {
  border-color: rgba(248, 113, 113, .42);
  background: linear-gradient(180deg, #ef4444, #991b1b);
  color: #fff7ed;
}
.btn.is-done {
  border-color: rgba(146, 141, 255, .5);
  background: linear-gradient(180deg, #4642ff, #2d2baf);
}
.btn-secondary.is-done {
  background: rgba(70, 66, 255, .16);
  color: #d7d5ff;
}
.status-pill {
  border-color: rgba(115, 109, 255, .32);
  background: rgba(70, 66, 255, .16);
  color: #c7c4ff;
}
.settings-modal {
  background: rgba(5, 5, 10, .78);
}
.settings-dialog {
  background: #1c1b2b;
}
.settings-grid select {
  background-color: var(--surface-control);
  color: var(--ink);
}
.settings-feedback {
  border-color: var(--theme-soft-line);
  background: rgba(15, 23, 42, .72);
  color: var(--ink-soft);
}
.settings-feedback.success {
  border-color: rgba(115, 109, 255, .36);
  background: rgba(70, 66, 255, .14);
  color: #c7c4ff;
}
.settings-feedback.error {
  border-color: rgba(248, 113, 113, .34);
  background: rgba(127, 29, 29, .22);
  color: #fecaca;
}
.settings-feedback.working {
  border-color: rgba(245, 184, 75, .34);
  background: rgba(120, 53, 15, .18);
  color: #fde68a;
}
.device-group-title {
  color: #8f89ff;
  letter-spacing: .02em;
}
.device {
  position: relative;
  border-color: var(--theme-soft-line);
  background: linear-gradient(180deg, rgba(22, 30, 44, .96), rgba(13, 18, 28, .96));
  color: var(--ink);
  box-shadow: 0 12px 28px rgba(0, 0, 0, .18);
  overflow: hidden;
}
.device::before {
  content: "";
  position: absolute;
  inset: 0 auto 0 0;
  width: 3px;
  background: rgba(148, 163, 184, .28);
}
.device:hover {
  border-color: rgba(115, 109, 255, .36);
  background: linear-gradient(180deg, rgba(39, 38, 58, .98), rgba(20, 19, 32, .98));
}
.device.active {
  border-color: rgba(115, 109, 255, .66);
  background: linear-gradient(180deg, rgba(70, 66, 255, .2), rgba(20, 19, 32, .96));
  outline: 2px solid rgba(70, 66, 255, .22);
}
.device.active::before {
  background: linear-gradient(180deg, #736dff, #4642ff);
}
.device strong {
  color: var(--ink);
}
.device-action {
  min-height: 32px;
  color: var(--ink-soft);
}
.device-action.start {
  border-color: rgba(115, 109, 255, .34);
  color: #c7c4ff;
}
.device-action.stop {
  border-color: rgba(248, 113, 113, .34);
  color: #fecaca;
}
.screen-wrap {
  border-color: rgba(111, 107, 255, .18);
  background:
    radial-gradient(circle at 50% 15%, rgba(70, 66, 255, .08), transparent 38%),
    linear-gradient(180deg, rgba(16, 15, 26, .36), rgba(0, 0, 0, .4)),
    #07070d;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, .04);
}
.empty-screen {
  color: rgba(203, 213, 225, .7);
}
.device-nav {
  border-top-color: rgba(111, 107, 255, .18);
  background: #0b0a12;
}
.device-nav button {
  color: #94a3b8;
}
.device-nav button:hover {
  background: rgba(70, 66, 255, .16);
  color: #d7d5ff;
}
.field-band {
  border-color: var(--theme-soft-line);
  background: rgba(15, 23, 42, .5);
}
.fullscreen-exit {
  border-color: rgba(115, 109, 255, .24);
  background: rgba(8, 7, 13, .82);
  color: #ecebff;
}
@media (max-width: 920px) {
  .sidebar-backdrop {
    background: rgba(2, 6, 23, .66);
  }
  .workspace-rail {
    border-color: rgba(115, 109, 255, .2);
  }
}
@media (max-width: 560px) {
  body,
  .app-shell {
    background: #08070d;
  }
  .monitor-card,
  .logs-panel,
  .workspace-rail,
  .app-controls {
    border-color: var(--theme-soft-line);
  }
}

/* Minimal information layer. Keep controls visible, remove ornamental weight. */
:root {
  --theme-line: rgba(111, 107, 255, .24);
  --theme-soft-line: rgba(255, 255, 255, .075);
  --surface: #171622;
  --surface-strong: #1f1e2d;
  --surface-muted: #202033;
  --surface-control: #11101b;
  --ink: #f7f7ff;
  --ink-muted: #a19fbd;
  --ink-soft: #d8d6f2;
  --shadow-panel: none;
  --shadow-soft: none;
}
.app-shell {
  background:
    radial-gradient(circle at 68% 8%, rgba(70, 66, 255, .16), transparent 30%),
    #08070d;
}
.app-layout {
  grid-template-columns: 300px minmax(420px, 1fr);
  gap: 10px;
  padding: 10px;
}
.workspace-rail,
.app-controls,
.monitor-card,
.settings-dialog {
  background: var(--surface);
  box-shadow: none;
  backdrop-filter: none;
}
.logs-panel {
  display: none;
  background: var(--surface);
  box-shadow: none;
}
.logs-panel.visible {
  display: block;
}
.workspace-rail,
.app-controls {
  min-height: calc(100vh - 20px);
}
.app-controls {
  position: fixed;
  top: 10px;
  right: 10px;
  bottom: 10px;
  z-index: 58;
  width: min(320px, calc(100vw - 20px));
  display: none;
  overflow: auto;
}
.app-controls.open {
  display: block;
}
.brand-lockup,
.monitor-toolbar {
  background: transparent;
}
.brand-lockup {
  padding: 14px;
}
.brand-mark {
  width: 32px;
  height: 32px;
  border-radius: 7px;
  box-shadow: none;
}
.brand-title {
  font-size: 15px;
}
.panel-section,
.monitor-card,
.logs-panel {
  padding: 12px;
}
.section-head,
.monitor-toolbar {
  margin-bottom: 10px;
}
.monitor-toolbar {
  padding-bottom: 10px;
}
.section-title {
  font-size: 13px;
}
.monitor-title {
  font-size: 18px;
}
.section-kicker,
.brand-subtitle {
  display: none;
}
#selectedMeta,
#status {
  font-size: 11px;
}
.status-pill {
  display: none;
}
.toolbar-actions {
  gap: 6px;
}
.icon-btn,
.btn,
.device-action {
  min-height: 36px;
  border-radius: 7px;
  background: var(--surface-control);
  box-shadow: none;
}
.icon-btn {
  width: 36px;
  height: 36px;
}
.btn {
  padding: 8px 12px;
  border-color: rgba(115, 109, 255, .42);
  background: #4642ff;
}
.btn-secondary {
  background: var(--surface-control);
  border-color: var(--theme-line);
}
.btn-danger {
  border-color: rgba(255, 103, 131, .32);
  background: #7f1d3a;
}
.icon-btn:hover,
.btn:hover,
.device-action:hover {
  transform: none;
  background: #24233a;
}
.device-list,
.device-group {
  gap: 6px;
}
.device-group + .device-group {
  margin-top: 10px;
}
.device-group-title {
  font-size: 10px;
  color: var(--ink-muted);
}
.device {
  min-height: 54px;
  padding: 10px 10px 10px 12px;
  background: var(--surface-muted);
  box-shadow: none;
}
.device::before {
  width: 2px;
}
.device-row {
  align-items: center;
}
.device-main {
  gap: 2px;
}
.device strong {
  font-size: 13px;
  line-height: 1.2;
}
.muted {
  font-size: 11px;
}
.device-action {
  min-height: 30px;
  padding: 4px 9px;
  font-size: 11px;
}
.screen-wrap {
  min-height: min(72vh, 700px);
  background: #07070d;
  box-shadow: none;
}
#screenCanvas,
#screenStream,
#screenVideo {
  max-height: min(72vh, 700px);
}
.device-nav {
  height: 40px;
  background: #0d0c15;
}
.field-band {
  padding: 10px;
  gap: 8px;
  background: var(--surface-muted);
}
.settings-dialog {
  width: min(300px, calc(100vw - 32px));
}
.settings-grid,
.settings-actions {
  gap: 8px;
}
.settings-feedback {
  min-height: 34px;
  margin-top: 10px;
}
@media (max-width: 1240px) {
  .app-layout {
    grid-template-columns: 280px minmax(0, 1fr);
  }
  .app-controls {
    min-height: 0;
  }
}
@media (max-width: 920px) {
  html,
  body,
  .app-shell,
  .app-layout,
  .monitor-pane,
  .monitor-card,
  .app-controls {
    width: 100%;
    max-width: 100vw;
    overflow-x: hidden;
  }
  .app-layout {
    grid-template-columns: minmax(0, 1fr);
    padding: 0;
    gap: 0;
  }
  .workspace-rail {
    width: min(82vw, 300px);
  }
  .app-controls {
    top: auto;
    left: 0;
    right: 0;
    bottom: 0;
    width: 100%;
    max-height: min(64vh, 420px);
    min-height: 0;
    border-radius: 10px 10px 0 0;
    border-left: 0;
    border-right: 0;
    border-bottom: 0;
  }
  .monitor-card {
    border-radius: 0;
    border-left: 0;
    border-right: 0;
  }
  .screen-wrap,
  .device-nav {
    width: 100%;
    min-width: 0;
  }
}
@media (max-width: 560px) {
  .monitor-toolbar {
    gap: 8px;
  }
  .toolbar-actions {
    display: flex;
    max-width: calc(100vw - 24px);
    overflow: hidden;
  }
  .toolbar-actions .icon-btn {
    flex: 0 0 40px;
    width: 40px;
  }
  .toolbar-actions .btn {
    flex: 1 1 0;
    min-width: 0;
    width: auto;
    padding-left: 6px;
    padding-right: 6px;
  }
  .toolbar-actions #settingsOpen {
    display: none;
  }
  .screen-wrap {
    min-height: 62vh;
  }
  #screenCanvas,
  #screenStream,
  #screenVideo {
    max-height: 62vh;
  }
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
  emulatorRtcSocket: null,
  webrtcSession: null,
  webrtcMode: null,
  webrtcFrames: new Map(),
  webrtcWatchdog: null,
  webrtcFramePoll: null,
  videoFrameCallbackId: null,
  latestWebRtcFrameSeq: 0,
  latestVideoFrameAt: 0,
  previewUrl: null,
  previewSeq: 0,
  pointerStart: null,
  pointerStream: null,
  pending: new Map(),
  requestSeq: 0,
  feedbackTimer: null
};
const el = id => document.getElementById(id);

function setStatus(text) {
  el('status').textContent = text || '';
  el('statusChip').textContent = text ? 'Active' : 'Idle';
}
function setSettingsFeedback(text, kind = 'success') {
  const feedback = el('settingsFeedback');
  if (!feedback) return;
  clearTimeout(state.feedbackTimer);
  feedback.textContent = text || 'Ready';
  feedback.classList.remove('success', 'error', 'working');
  if (kind) feedback.classList.add(kind);
  if (kind === 'success') {
    state.feedbackTimer = setTimeout(() => {
      feedback.textContent = 'Ready';
      feedback.classList.remove('success', 'error', 'working');
    }, 2200);
  }
}
function updateSettingsControlVisibility() {
  const mode = el('viewMode').value;
  const showField = (id, visible) => {
    const field = el(id);
    if (!field) return;
    field.style.display = visible ? '' : 'none';
  };
  showField('pollFpsField', mode === 'poll');
  showField('streamFpsField', mode !== 'poll');
  showField('streamFormatField', mode !== 'poll');
  showField('streamScaleField', mode !== 'poll');
  showField('streamQualityField', mode !== 'poll');
}
async function withSettingsButtonFeedback(buttonId, workingText, doneText, action) {
  const button = el(buttonId);
  const originalText = button.textContent;
  button.classList.remove('is-done');
  button.classList.add('is-working');
  button.disabled = true;
  button.textContent = workingText;
  setSettingsFeedback(workingText, 'working');
  try {
    await action();
    button.classList.remove('is-working');
    button.classList.add('is-done');
    button.textContent = doneText;
    setSettingsFeedback(doneText, 'success');
    setTimeout(() => {
      button.classList.remove('is-done');
      button.disabled = false;
      button.textContent = originalText;
    }, 900);
  } catch (err) {
    button.classList.remove('is-working');
    button.disabled = false;
    button.textContent = originalText;
    setSettingsFeedback(err.message || 'Action failed', 'error');
    setStatus(err.message);
  }
}
function settingsControlUpdated(text) {
  setSettingsFeedback(text, 'success');
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
function devicePlatform(device) {
  const kind = (device && device.kind ? device.kind : '').toLowerCase();
  if (kind.includes('android')) return 'android';
  if (kind.includes('ios')) return 'ios';
  return 'unknown';
}
function selectedPlatform() {
  return state.selected ? devicePlatform(state.selected) : 'unknown';
}
function effectiveStreamSettings() {
  const requestedFormat = el('streamFormat').value;
  const format = requestedFormat === 'native' && selectedPlatform() !== 'android'
    ? 'auto'
    : requestedFormat;
  const mode = el('viewMode').value;
  const selectedFps = Number(el('streamFps').value) || 8;
  const fps = mode === 'stream'
    ? Math.min(selectedFps, 15)
    : Math.max(selectedFps, 12);
  return {
    fps,
    format,
    maxWidth: Number(el('streamScale').value) || 720,
    quality: Number(el('streamQuality').value) || 70,
    label: `${format === 'auto' ? 'APPMON' : format.toUpperCase()} ${fps}FPS`
  };
}
function selectDevice(deviceId) {
  const device = state.devices.find(item => item.id === deviceId);
  if (!device) return;
  state.selected = device;
  renderDevices();
  updateSettingsControlVisibility();
  restartPreview();
  if (isMobileSidebar()) closeSidebar();
}
function isDeviceRunning(device) {
  const stateText = (device && device.state ? device.state : '').toLowerCase();
  return stateText === 'booted' || stateText === 'device';
}
function deviceActionLabel(device) {
  return isDeviceRunning(device) ? 'Stop' : 'Start';
}
function compactDeviceState(device) {
  const stateText = device && device.state ? device.state : 'Unknown';
  return stateText.charAt(0).toUpperCase() + stateText.slice(1);
}
async function setDevicePower(device, actionButton) {
  const action = isDeviceRunning(device) ? 'stop' : 'start';
  actionButton.disabled = true;
  actionButton.textContent = action === 'start' ? 'Starting' : 'Stopping';
  try {
    await post(`/api/devices/${encodeURIComponent(device.id)}/${action}`, {});
    setStatus(`${action === 'start' ? 'Started' : 'Stopped'} ${device.name}`);
    await loadDevices();
  } catch (err) {
    setStatus(err.message);
    renderDevices();
  }
}
function renderDevices() {
  el('devices').innerHTML = '';
  el('selectedMeta').textContent = state.selected
    ? `${state.selected.name} / ${compactDeviceState(state.selected)}`
    : 'No device selected';
  updateDeviceNav();
  for (const [platform, title] of [['android', 'Android'], ['ios', 'iOS']]) {
    const devices = state.devices.filter(device => devicePlatform(device) === platform);
    if (!devices.length) continue;

    const group = document.createElement('section');
    group.className = 'device-group';
    const heading = document.createElement('h3');
    heading.className = 'device-group-title';
    heading.textContent = title;
    group.appendChild(heading);

    for (const device of devices) {
      const btn = document.createElement('div');
      btn.className = `device ${state.selected && state.selected.id === device.id ? 'active' : ''}`;
      btn.role = 'button';
      btn.tabIndex = 0;
      btn.onclick = () => selectDevice(device.id);
      btn.onkeydown = event => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          selectDevice(device.id);
        }
      };

      const row = document.createElement('div');
      row.className = 'device-row';
      const main = document.createElement('div');
      main.className = 'device-main';
      const name = document.createElement('strong');
      name.textContent = device.name;
      const meta = document.createElement('span');
      meta.className = 'muted';
      meta.textContent = compactDeviceState(device);
      main.append(name, meta);

      const actions = document.createElement('div');
      actions.className = 'device-actions';
      const action = document.createElement('button');
      action.type = 'button';
      action.className = `device-action ${isDeviceRunning(device) ? 'stop' : 'start'}`;
      action.textContent = deviceActionLabel(device);
      action.onclick = event => {
        event.preventDefault();
        event.stopPropagation();
        setDevicePower(device, action);
      };
      actions.appendChild(action);
      row.append(main, actions);
      btn.appendChild(row);
      group.appendChild(btn);
    }

    el('devices').appendChild(group);
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
    const image = await preloadImage(url);
    if (seq !== state.previewSeq) {
      URL.revokeObjectURL(url);
      return;
    }
    showPreviewImage(image, url);
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
  const stream = effectiveStreamSettings();
  const params = new URLSearchParams({
    fps: stream.fps.toString(),
    format: stream.format,
    max_width: stream.maxWidth.toString(),
    quality: stream.quality.toString(),
    t: Date.now().toString()
  });
  const path = `/api/devices/${selectedId()}/screenshot-stream?${params}`;
  if (isAndroidSelected() && (stream.format === 'auto' || stream.format === 'video')) {
    showStreamImage(path, seq);
    setStatus(`Fast Android video stream / ${stream.label}`);
    return;
  }
  readScreenshotStream(path, controller.signal, seq).catch(err => {
    if (err.name !== 'AbortError') setStatus(err.message);
  });
  setStatus(`Streaming ${stream.fps} fps / ${stream.label}`);
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
  const stream = effectiveStreamSettings();
  if (isAndroidSelected() && stream.format === 'native') {
    try {
      await startNativeEmulatorWebRtcStream(seq);
      return;
    } catch (err) {
      if (seq !== state.previewSeq) return;
      stopWebRtc();
      const message = `Native WebRTC unavailable, using Appmon video: ${err.message}`;
      setStatus(message);
      setSettingsFeedback(message, 'working');
    }
  }
  await startAppmonWebRtcStream(seq);
}
async function startAppmonWebRtcStream(seq) {
  const stream = effectiveStreamSettings();
  if (stream.format !== 'video') {
    try {
      await startWebRtcDataStream(seq);
    } catch (dataErr) {
      if (seq !== state.previewSeq) return;
      setStatus(`WebRTC unavailable: ${dataErr.message}`);
      startScreenshotStream();
    }
    return;
  }

  await startWebRtcMediaFirstStream(seq);
}
async function startWebRtcMediaFirstStream(seq) {
  const mediaTransports = [
    { transport: 'media_h264', label: 'H.264' },
    { transport: 'media_vp8', label: 'VP8' }
  ];
  let mediaErr = null;
  for (const media of mediaTransports) {
    try {
      await startWebRtcMediaStream(seq, media);
      return;
    } catch (err) {
      if (seq !== state.previewSeq) return;
      mediaErr = err;
      stopWebRtc();
      setStatus(`WebRTC ${media.label} unavailable: ${err.message}`);
    }
  }

  try {
    await startWebRtcDataStream(seq);
  } catch (dataErr) {
    if (seq !== state.previewSeq) return;
    const mediaMessage = mediaErr ? `${mediaErr.message}; ` : '';
    setStatus(`WebRTC unavailable: ${mediaMessage}${dataErr.message}`);
    startScreenshotStream();
  }
}
function startNativeEmulatorWebRtcStream(seq) {
  return new Promise((resolve, reject) => {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const socket = new WebSocket(`${proto}://${location.host}/api/devices/${selectedId()}/emulator-webrtc/ws`);
    const patch = { candidates: [], sdp: null, haveOffer: false, answer: false };
    let settled = false;
    let peer = null;
    const timeout = setTimeout(() => {
      settleReject(new Error('native emulator WebRTC video timed out'));
    }, 3000);
    const settleResolve = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      resolve();
    };
    const settleReject = err => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      reject(err instanceof Error ? err : new Error(String(err)));
    };
    const sendSignal = value => {
      if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify(value));
    };
    const handleCandidate = signal => {
      if (!peer) return;
      peer.addIceCandidate(new RTCIceCandidate(signal)).catch(err => {
        if (seq === state.previewSeq) setStatus(`Native ICE failed: ${err.message}`);
      });
    };
    const handleSdp = async signal => {
      if (!peer || patch.answer) return;
      await peer.setRemoteDescription(new RTCSessionDescription(signal));
      const answer = await peer.createAnswer();
      if (!answer || patch.answer) return;
      patch.answer = true;
      await peer.setLocalDescription(answer);
      sendSignal({ sdp: answer });
    };
    const flushSignals = () => {
      if (!peer) return;
      if (patch.sdp) {
        const sdp = patch.sdp;
        patch.sdp = null;
        handleSdp(sdp).catch(settleReject);
      }
      if (patch.haveOffer) {
        while (patch.candidates.length) handleCandidate(patch.candidates.shift());
      }
    };
    const handleSignal = signal => {
      if (signal.error) {
        settleReject(new Error(signal.error));
        return;
      }
      if (signal.start) {
        peer = new RTCPeerConnection(signal.start);
        state.webrtcPeer = peer;
        state.webrtcMode = 'native';
        state.webrtcFrames.clear();
        peer.ontrack = event => {
          if (seq !== state.previewSeq) return;
          const video = el('screenVideo');
          el('screenStream').style.display = 'none';
          video.srcObject = event.streams && event.streams.length
            ? event.streams[0]
            : new MediaStream([event.track]);
          video.style.display = 'block';
          el('screenCanvas').style.display = 'none';
          el('screenEmpty').style.display = 'none';
          trackVideoFrames(video, seq);
          startWebRtcVideoWatchdog(seq, 'Native WebRTC', () => {
            setStatus('Native WebRTC stalled, using Appmon video');
            stopWebRtc();
            startAppmonWebRtcStream(++state.previewSeq).catch(err => {
              if (el('viewMode').value === 'webrtc') {
                setStatus(`WebRTC unavailable: ${err.message}`);
                startScreenshotStream();
              }
            });
          });
          video.play().catch(() => {});
          setStatus('Native emulator WebRTC video');
          setSettingsFeedback('Native emulator WebRTC video connected', 'success');
          settleResolve();
        };
        peer.onicecandidate = event => {
          if (event.candidate) sendSignal({ candidate: event.candidate });
        };
        peer.ondatachannel = event => {
          // The emulator may expose input data channels. Appmon keeps input on
          // its existing control path so preview and control can fail over
          // independently.
          event.channel.binaryType = 'arraybuffer';
        };
        peer.onsignalingstatechange = () => {
          if (peer && peer.signalingState === 'have-remote-offer') {
            patch.haveOffer = true;
            flushSignals();
          }
        };
        peer.onconnectionstatechange = () => {
          if (seq !== state.previewSeq || !peer) return;
          if (peer.connectionState === 'connected' && !settled) {
            setStatus('Native emulator WebRTC connected, waiting for video');
            setSettingsFeedback('Native WebRTC connected, waiting for video track', 'working');
          }
          if (peer.connectionState === 'failed' || peer.connectionState === 'disconnected') {
            if (!settled) settleReject(new Error(`native emulator WebRTC ${peer.connectionState}`));
            else {
              setStatus('Native WebRTC lost, using Appmon video');
              stopWebRtc();
              startAppmonWebRtcStream(++state.previewSeq).catch(err => {
                if (el('viewMode').value === 'webrtc') {
                  setStatus(`WebRTC unavailable: ${err.message}`);
                  startScreenshotStream();
                }
              });
            }
          }
        };
      }
      if (signal.bye) {
        if (!settled) settleReject(new Error('native emulator WebRTC closed'));
        return;
      }
      if (signal.sdp && !patch.sdp) patch.sdp = signal;
      if (signal.candidate) patch.candidates.push(signal);
      flushSignals();
    };
    socket.onopen = () => {
      if (seq === state.previewSeq) {
        setStatus('Native emulator WebRTC connecting');
        setSettingsFeedback('Requesting native emulator WebRTC video', 'working');
      }
    };
    socket.onmessage = event => {
      if (seq !== state.previewSeq) return;
      try {
        handleSignal(JSON.parse(event.data));
      } catch (err) {
        settleReject(err);
      }
    };
    socket.onerror = () => settleReject(new Error('native emulator WebRTC socket failed'));
    socket.onclose = () => {
      if (!settled) settleReject(new Error('native emulator WebRTC socket closed'));
    };
    state.emulatorRtcSocket = socket;
  });
}
function trackVideoFrames(video, seq) {
  state.latestVideoFrameAt = performance.now();
  if (state.videoFrameCallbackId && video.cancelVideoFrameCallback) {
    try { video.cancelVideoFrameCallback(state.videoFrameCallbackId); } catch (_) {}
  }
  state.videoFrameCallbackId = null;
  if (video.requestVideoFrameCallback) {
    const onFrame = () => {
      if (seq !== state.previewSeq) return;
      state.latestVideoFrameAt = performance.now();
      state.videoFrameCallbackId = video.requestVideoFrameCallback(onFrame);
    };
    state.videoFrameCallbackId = video.requestVideoFrameCallback(onFrame);
    return;
  }
  let lastTime = video.currentTime;
  state.webrtcFramePoll = setInterval(() => {
    if (seq !== state.previewSeq) return;
    if (video.readyState >= 2 && video.currentTime !== lastTime) {
      lastTime = video.currentTime;
      state.latestVideoFrameAt = performance.now();
    }
  }, 500);
}
function startWebRtcVideoWatchdog(seq, label, onStalled) {
  clearInterval(state.webrtcWatchdog);
  state.webrtcWatchdog = setInterval(() => {
    if (seq !== state.previewSeq || el('viewMode').value !== 'webrtc') return;
    const video = el('screenVideo');
    const stalledFor = performance.now() - state.latestVideoFrameAt;
    if (video.style.display !== 'none' && stalledFor > 3500) {
      clearInterval(state.webrtcWatchdog);
      state.webrtcWatchdog = null;
      setSettingsFeedback(`${label} stalled; switching transport`, 'working');
      onStalled();
    }
  }, 1000);
}
async function startWebRtcMediaStream(seq, media) {
  const peer = new RTCPeerConnection({ iceServers: [] });
  state.webrtcPeer = peer;
  state.webrtcMode = media.transport;
  state.webrtcFrames.clear();
  let mediaReady = false;
  let mediaReadyTimer = null;
  let rejectMediaReady = () => {};
  const mediaReadyPromise = new Promise((resolve, reject) => {
    rejectMediaReady = reject;
    mediaReadyTimer = setTimeout(() => {
      if (!mediaReady) reject(new Error('WebRTC video did not receive frames'));
    }, media.timeoutMs || 5000);
    const resolveReady = () => {
      if (mediaReady) return;
      mediaReady = true;
      clearTimeout(mediaReadyTimer);
      resolve();
    };
    peer._appmonResolveMediaReady = resolveReady;
  });
  peer.addTransceiver('video', { direction: 'recvonly' });
  peer.ontrack = event => {
    if (seq !== state.previewSeq) return;
    const video = el('screenVideo');
    el('screenStream').style.display = 'none';
    video.srcObject = event.streams && event.streams.length
      ? event.streams[0]
      : new MediaStream([event.track]);
    video.style.display = 'block';
    el('screenCanvas').style.display = 'none';
    el('screenEmpty').style.display = 'none';
    trackVideoFrames(video, seq);
    startWebRtcVideoWatchdog(seq, `WebRTC ${media.label}`, () => {
      setStatus(`WebRTC ${media.label} stalled, using data channel`);
      stopWebRtc();
      startWebRtcDataStream(++state.previewSeq).catch(err => {
        if (el('viewMode').value === 'webrtc') {
          setStatus(`WebRTC unavailable: ${err.message}`);
          startScreenshotStream();
        }
      });
    });
    const markReady = () => {
      if (seq !== state.previewSeq) return;
      if (!video.videoWidth && video.readyState < 2) return;
      const stream = effectiveStreamSettings();
      setStatus(`WebRTC ${media.label} video ${stream.fps} fps`);
      setSettingsFeedback(`WebRTC ${media.label} video connected`, 'success');
      if (peer._appmonResolveMediaReady) peer._appmonResolveMediaReady();
    };
    video.onloadedmetadata = markReady;
    video.onloadeddata = markReady;
    video.onplaying = markReady;
    video.play().catch(() => {});
    markReady();
  };
  peer.onconnectionstatechange = () => {
    if (seq !== state.previewSeq) return;
    if (peer.connectionState === 'failed' || peer.connectionState === 'disconnected') {
      if (!mediaReady) {
        rejectMediaReady(new Error(`WebRTC video ${peer.connectionState}`));
        return;
      }
      setStatus(`WebRTC ${media.label} lost, using data channel`);
      stopWebRtc();
      startWebRtcDataStream(++state.previewSeq).catch(err => {
        if (el('viewMode').value === 'webrtc') {
          setStatus(`WebRTC unavailable: ${err.message}`);
          startScreenshotStream();
        }
      });
    }
  };

  const offer = await peer.createOffer();
  await peer.setLocalDescription(offer);
  await waitForIceGathering(peer);
  if (seq !== state.previewSeq) return;
  const response = await postWebRtcOffer(media.transport, peer.localDescription);
  if (seq !== state.previewSeq) return;
  state.webrtcSession = response.session_id;
  await peer.setRemoteDescription(response.answer);
  setStatus(`WebRTC ${media.label} video connecting`);
  await mediaReadyPromise;
}
async function startWebRtcDataStream(seq) {
  if (seq !== state.previewSeq) return;
  const peer = new RTCPeerConnection({ iceServers: [] });
  const channel = peer.createDataChannel('appmon-preview', {
    ordered: false,
    maxRetransmits: 0
  });
  channel.binaryType = 'arraybuffer';
  state.webrtcPeer = peer;
  state.webrtcChannel = channel;
  state.webrtcMode = 'data';
  state.webrtcFrames.clear();

  channel.onopen = () => {
    const stream = effectiveStreamSettings();
    setStatus(`WebRTC ${stream.fps} fps / ${stream.label}`);
    setSettingsFeedback('WebRTC data channel connected', 'success');
  };
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
    const response = await postWebRtcOffer('data', peer.localDescription);
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
function postWebRtcOffer(transport, offer) {
  const stream = effectiveStreamSettings();
  const params = {
    transport,
    fps: stream.fps,
    format: stream.format,
    max_width: stream.maxWidth,
    quality: stream.quality,
    offer
  };
  return json(`/api/devices/${selectedId()}/webrtc/offer`, {
    method: 'POST',
    body: JSON.stringify(params)
  });
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
  clearInterval(state.webrtcWatchdog);
  clearInterval(state.webrtcFramePoll);
  state.webrtcWatchdog = null;
  state.webrtcFramePoll = null;
  if (state.emulatorRtcSocket) {
    state.emulatorRtcSocket.onopen = null;
    state.emulatorRtcSocket.onmessage = null;
    state.emulatorRtcSocket.onerror = null;
    state.emulatorRtcSocket.onclose = null;
    try { state.emulatorRtcSocket.close(); } catch (_) {}
    state.emulatorRtcSocket = null;
  }
  state.webrtcFrames.clear();
  state.latestWebRtcFrameSeq = 0;
  state.webrtcSession = null;
  state.webrtcMode = null;
  const video = el('screenVideo');
  const canvas = el('screenCanvas');
  const streamImage = el('screenStream');
  if (state.videoFrameCallbackId && video.cancelVideoFrameCallback) {
    try { video.cancelVideoFrameCallback(state.videoFrameCallbackId); } catch (_) {}
  }
  state.videoFrameCallbackId = null;
  state.latestVideoFrameAt = 0;
  streamImage.onload = null;
  streamImage.onerror = null;
  streamImage.removeAttribute('src');
  streamImage.style.display = 'none';
  video.pause();
  video.onloadedmetadata = null;
  video.onloadeddata = null;
  video.onplaying = null;
  video.removeAttribute('src');
  video.srcObject = null;
  video.style.display = 'none';
  canvas.style.display = 'none';
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
      const image = await preloadImage(url);
      if (seq !== state.previewSeq) {
        URL.revokeObjectURL(url);
        return;
      }
      showPreviewImage(image, url);
      await nextAnimationFrame();
    }
  }
}
function waitForIceGathering(peer) {
  if (peer.iceGatheringState === 'complete') return Promise.resolve();
  return new Promise(resolve => {
    const timeout = setTimeout(done, 250);
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
  if (frameSeq <= state.latestWebRtcFrameSeq) return;
  const merged = new Uint8Array(frame.totalLength || frame.received);
  let offset = 0;
  for (const part of frame.chunks) {
    merged.set(part.subarray(0, Math.min(part.length, merged.length - offset)), offset);
    offset += part.length;
    if (offset >= merged.length) break;
  }
  const url = URL.createObjectURL(new Blob([merged], { type: frame.contentType }));
  const image = await preloadImage(url);
  if (seq !== state.previewSeq) {
    URL.revokeObjectURL(url);
    return;
  }
  if (frameSeq <= state.latestWebRtcFrameSeq) {
    URL.revokeObjectURL(url);
    return;
  }
  state.latestWebRtcFrameSeq = frameSeq;
  showPreviewImage(image, url);
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
  if (image.decode) return image.decode().then(() => image);
  return new Promise((resolve, reject) => {
    image.onload = () => resolve(image);
    image.onerror = reject;
  });
}
function showPreviewImage(image, url) {
  const canvas = el('screenCanvas');
  const context = canvas.getContext('2d');
  const video = el('screenVideo');
  const streamImage = el('screenStream');
  const previousUrl = state.previewUrl;
  state.previewUrl = url;
  streamImage.removeAttribute('src');
  streamImage.style.display = 'none';
  video.pause();
  video.srcObject = null;
  video.style.display = 'none';
  canvas.width = image.naturalWidth || image.width;
  canvas.height = image.naturalHeight || image.height;
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.drawImage(image, 0, 0, canvas.width, canvas.height);
  canvas.style.display = 'block';
  el('screenEmpty').style.display = 'none';
  if (previousUrl) requestAnimationFrame(() => URL.revokeObjectURL(previousUrl));
}
function showStreamImage(path, seq) {
  const streamImage = el('screenStream');
  const video = el('screenVideo');
  const canvas = el('screenCanvas');
  video.pause();
  video.srcObject = null;
  video.style.display = 'none';
  canvas.style.display = 'none';
  streamImage.onload = () => {
    if (seq !== state.previewSeq) return;
    el('screenEmpty').style.display = 'none';
  };
  streamImage.onerror = () => {
    if (seq === state.previewSeq) setStatus('Fast Android stream stopped');
  };
  streamImage.src = path;
  streamImage.style.display = 'block';
  el('screenEmpty').style.display = 'none';
}
function restartPreview() {
  updateDeviceNav();
  updateSettingsControlVisibility();
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
  updateSettingsControlVisibility();
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
function openAppPanel() {
  const panel = el('appPanel');
  panel.classList.add('open');
  panel.setAttribute('aria-hidden', 'false');
  el('appId').focus();
}
function closeAppPanel() {
  const panel = el('appPanel');
  panel.classList.remove('open');
  panel.setAttribute('aria-hidden', 'true');
  el('appOpen').focus();
}
function isMobileSidebar() {
  return window.matchMedia('(max-width: 920px)').matches;
}
function setSidebarOpen(open) {
  document.body.classList.toggle('sidebar-open', open);
  document.body.classList.toggle('sidebar-lock', open);
  el('sidebarBackdrop').setAttribute('aria-hidden', open ? 'false' : 'true');
  el('sidebarToggle').setAttribute('aria-expanded', open ? 'true' : 'false');
}
function openSidebar() {
  if (!isMobileSidebar()) return;
  setSidebarOpen(true);
}
function closeSidebar() {
  setSidebarOpen(false);
}
function isPreviewFullscreen() {
  const activeElement = document.fullscreenElement || document.webkitFullscreenElement;
  const preview = el('screenWrap');
  return activeElement === preview || preview.classList.contains('fallback-fullscreen');
}
function updateFullscreenButton() {
  const button = el('fullscreenToggle');
  const active = isPreviewFullscreen();
  button.title = active ? 'Exit fullscreen' : 'Enter fullscreen';
  button.setAttribute('aria-label', button.title);
  button.classList.toggle('active', active);
  if (!active) document.body.classList.remove('preview-fullscreen-lock');
}
async function exitPreviewFullscreen() {
  const preview = el('screenWrap');
  const exitFullscreen = document.exitFullscreen || document.webkitExitFullscreen;
  if (preview.classList.contains('fallback-fullscreen')) {
    preview.classList.remove('fallback-fullscreen');
    document.body.classList.remove('preview-fullscreen-lock');
  } else if ((document.fullscreenElement || document.webkitFullscreenElement) && exitFullscreen) {
    await exitFullscreen.call(document);
  }
  updateFullscreenButton();
}
async function toggleFullscreen() {
  const preview = el('screenWrap');
  const requestFullscreen = preview && (preview.requestFullscreen || preview.webkitRequestFullscreen);
  const exitFullscreen = document.exitFullscreen || document.webkitExitFullscreen;
  const fullscreenEnabled = document.fullscreenEnabled || document.webkitFullscreenEnabled;
  if (!preview) {
    return;
  }
  if (preview.classList.contains('fallback-fullscreen')) {
    await exitPreviewFullscreen();
    return;
  }
  if (isPreviewFullscreen()) {
    await exitPreviewFullscreen();
  } else if (fullscreenEnabled && requestFullscreen && exitFullscreen) {
    await requestFullscreen.call(preview);
  } else {
    preview.classList.add('fallback-fullscreen');
    document.body.classList.add('preview-fullscreen-lock');
    setStatus('Fullscreen preview');
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
async function controlQuiet(type, payload, restPath, restBody = payload) {
  const wsPromise = sendWsControl(type, payload);
  if (wsPromise) {
    await wsPromise;
    return;
  }
  await json(restPath, { method: 'POST', body: JSON.stringify(restBody) });
}
function imagePoint(event) {
  return clientPointToImage(event.clientX, event.clientY);
}
function pagePointToImage(pageX, pageY) {
  return clientPointToImage(pageX - window.scrollX, pageY - window.scrollY);
}
function clientPointToImage(clientX, clientY) {
  const canvas = el('screenCanvas');
  const streamImage = el('screenStream');
  const video = el('screenVideo');
  const usingVideo = video.style.display !== 'none' && video.videoWidth && video.videoHeight;
  const usingStream = !usingVideo && streamImage.style.display !== 'none' && streamImage.naturalWidth && streamImage.naturalHeight;
  const preview = usingVideo ? video : usingStream ? streamImage : canvas;
  const naturalWidth = usingVideo ? video.videoWidth : usingStream ? streamImage.naturalWidth : canvas.width;
  const naturalHeight = usingVideo ? video.videoHeight : usingStream ? streamImage.naturalHeight : canvas.height;
  if (!naturalWidth || !naturalHeight) throw new Error('Screenshot is not ready');
  const rect = preview.getBoundingClientRect();
  const x = Math.max(0, Math.min(rect.width, clientX - rect.left));
  const y = Math.max(0, Math.min(rect.height, clientY - rect.top));
  return {
    x: Math.round(x * naturalWidth / rect.width),
    y: Math.round(y * naturalHeight / rect.height),
    source_width: naturalWidth,
    source_height: naturalHeight
  };
}
function isAndroidSelected() {
  return selectedPlatform() === 'android';
}
function sendPointerMotion(action, point) {
  const payload = { action, ...point };
  return controlQuiet('motion', payload, `/api/devices/${selectedId()}/input/motion`);
}
function clearPointerStream() {
  if (state.pointerStream && state.pointerStream.moveTimer) {
    clearTimeout(state.pointerStream.moveTimer);
  }
  state.pointerStream = null;
}
function queuePointerMove(point) {
  const stream = state.pointerStream;
  if (!stream || !stream.active) return;
  stream.queuedMove = point;
  const now = performance.now();
  const elapsed = now - stream.lastMoveAt;
  const sendMove = () => {
    const current = state.pointerStream;
    if (!current || !current.active || !current.queuedMove) return;
    const move = current.queuedMove;
    current.queuedMove = null;
    current.moveTimer = null;
    current.lastMoveAt = performance.now();
    sendPointerMotion('move', move).catch(err => setStatus(err.message));
  };
  if (elapsed >= 16) {
    sendMove();
  } else if (!stream.moveTimer) {
    stream.moveTimer = setTimeout(sendMove, Math.max(1, 16 - elapsed));
  }
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
  el('logsPanel').classList.add('visible');
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
el('sidebarToggle').onclick = () => openSidebar();
el('sidebarBackdrop').onclick = () => closeSidebar();
document.addEventListener('click', event => {
  if (!document.body.classList.contains('sidebar-open')) return;
  const rail = document.querySelector('.workspace-rail');
  const toggle = el('sidebarToggle');
  if (rail.contains(event.target) || toggle.contains(event.target)) return;
  closeSidebar();
});
el('fullscreenToggle').onclick = () => toggleFullscreen().catch(err => setStatus(err.message));
el('fullscreenExit').onclick = () => exitPreviewFullscreen().catch(err => setStatus(err.message));
document.addEventListener('fullscreenchange', updateFullscreenButton);
document.addEventListener('webkitfullscreenchange', updateFullscreenButton);
el('settingsOpen').onclick = () => openSettings();
el('settingsClose').onclick = () => closeSettings();
el('settingsModal').onclick = event => {
  if (event.target === el('settingsModal')) closeSettings();
};
el('appOpen').onclick = () => openAppPanel();
el('appClose').onclick = () => closeAppPanel();
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && document.body.classList.contains('sidebar-open')) closeSidebar();
  if (event.key === 'Escape' && el('settingsModal').classList.contains('open')) closeSettings();
  if (event.key === 'Escape' && el('appPanel').classList.contains('open')) closeAppPanel();
  if (event.key === 'Escape' && el('screenWrap').classList.contains('fallback-fullscreen')) {
    exitPreviewFullscreen().catch(err => setStatus(err.message));
  }
});
window.addEventListener('resize', () => {
  if (!isMobileSidebar()) closeSidebar();
});
el('shot').onclick = () => withSettingsButtonFeedback('shot', 'Capturing...', 'Screenshot updated', async () => {
  if (!state.selected) throw new Error('Select a device first');
  await refreshScreenshot();
});
el('viewMode').onchange = () => {
  restartPreview();
  settingsControlUpdated(`Preview mode updated to ${el('viewMode').selectedOptions[0].textContent}`);
  updateSettingsControlVisibility();
};
el('pollFps').onchange = () => {
  if (el('viewMode').value === 'poll') startPolling();
  settingsControlUpdated(`Polling updated to ${el('pollFps').selectedOptions[0].textContent}`);
};
el('streamFps').onchange = () => {
  if (el('viewMode').value !== 'poll') restartPreview();
  settingsControlUpdated(`Stream FPS updated to ${el('streamFps').selectedOptions[0].textContent}`);
};
el('streamFormat').onchange = () => {
  if (el('viewMode').value !== 'poll') restartPreview();
  settingsControlUpdated(`Stream format updated to ${el('streamFormat').selectedOptions[0].textContent}`);
};
el('streamScale').onchange = () => {
  if (el('viewMode').value !== 'poll') restartPreview();
  settingsControlUpdated(`Stream scale updated to ${el('streamScale').selectedOptions[0].textContent}`);
};
el('streamQuality').onchange = () => {
  if (el('viewMode').value !== 'poll') restartPreview();
  settingsControlUpdated(`Quality updated to ${el('streamQuality').selectedOptions[0].textContent}`);
};
el('logsBtn').onclick = () => withSettingsButtonFeedback('logsBtn', 'Loading logs...', 'Logs updated', loadLogs);
el('navBack').onclick = () => sendKeyValue('BACK').catch(err => setStatus(err.message));
el('navHome').onclick = () => sendKeyValue('HOME').catch(err => setStatus(err.message));
el('navRecents').onclick = () => sendKeyValue('APP_SWITCH').catch(err => setStatus(err.message));
el('install').onclick = () => post(`/api/devices/${selectedId()}/app/install`, { path: el('appPath').value }).catch(err => setStatus(err.message));
el('launch').onclick = () => post(`/api/devices/${selectedId()}/app/launch`, { app_id: el('appId').value }).catch(err => setStatus(err.message));
el('terminate').onclick = () => post(`/api/devices/${selectedId()}/app/terminate`, { app_id: el('appId').value }).catch(err => setStatus(err.message));
el('recordStart').onclick = () => withSettingsButtonFeedback('recordStart', 'Starting...', 'Recording started', () => post(`/api/devices/${selectedId()}/record/start`, {}));
el('recordStop').onclick = () => withSettingsButtonFeedback('recordStop', 'Stopping...', 'Recording stopped', () => post(`/api/devices/${selectedId()}/record/stop`, {}));
function setupScreenControls() {
  const screen = el('screenWrap');
  document.body.dataset.touchBackend = 'pointer-stream';
  screen.addEventListener('pointerdown', event => {
    try {
      if (!state.selected) throw new Error('Select a device first');
      screen.setPointerCapture(event.pointerId);
      const point = imagePoint(event);
      if (isAndroidSelected()) {
        clearPointerStream();
        state.pointerStream = {
          active: true,
          pointerId: event.pointerId,
          start: point,
          lastMoveAt: performance.now(),
          queuedMove: null,
          moveTimer: null
        };
        sendPointerMotion('down', point).catch(err => setStatus(err.message));
      } else {
        state.pointerStart = point;
      }
      event.preventDefault();
    } catch (err) {
      setStatus(err.message);
    }
  });
  screen.addEventListener('pointermove', event => {
    try {
      if (!state.pointerStream || state.pointerStream.pointerId !== event.pointerId) return;
      if (!isAndroidSelected()) return;
      queuePointerMove(imagePoint(event));
      event.preventDefault();
    } catch (err) {
      setStatus(err.message);
    }
  });
  screen.addEventListener('pointerup', event => {
    try {
      if (state.pointerStream && state.pointerStream.pointerId === event.pointerId) {
        const stream = state.pointerStream;
        const end = imagePoint(event);
        if (stream.queuedMove) {
          sendPointerMotion('move', stream.queuedMove).catch(err => setStatus(err.message));
        }
        clearPointerStream();
        event.preventDefault();
        sendPointerMotion('up', end)
          .then(() => setStatus(`Pointer ${end.x}, ${end.y}`))
          .catch(err => setStatus(err.message));
        return;
      }
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
  screen.addEventListener('pointercancel', event => {
    if (state.pointerStream && state.pointerStream.pointerId === event.pointerId) {
      const point = state.pointerStream.queuedMove || state.pointerStream.start;
      clearPointerStream();
      sendPointerMotion('up', point).catch(err => setStatus(err.message));
    }
    state.pointerStart = null;
  });
}
setupScreenControls();
connectWs();
loadDevices().catch(err => setStatus(err.message));
"#;
