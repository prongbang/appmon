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
        div { class: "min-h-screen bg-coral text-slate-950",
            main { class: "grid min-h-screen grid-cols-app bg-white/95",
                DevicesPane {}
                MonitorPane {}
                AppControls {}
            }
        }
    }
}

#[component]
fn DevicesPane() -> Element {
    rsx! {
        aside { class: "border-r border-slate-200 bg-white/90 p-5",
            div { class: "mb-5 flex items-center justify-between gap-3",
                h2 { class: "text-base font-bold", "Devices" }
                button { id: "refresh", class: "btn btn-secondary", "Refresh" }
            }
            div { id: "devices", class: "grid gap-3" }
        }
    }
}

#[component]
fn MonitorPane() -> Element {
    rsx! {
        section { class: "overflow-auto bg-slate-50 p-6",
            div { class: "mb-5 flex justify-end",
                div { class: "flex flex-wrap items-center gap-2",
                    button { id: "shot", class: "btn btn-secondary", "Screenshot" }
                    button { id: "logsBtn", class: "btn btn-secondary", "Logs" }
                    button { id: "recordStart", class: "btn btn-secondary", "Record" }
                    button { id: "recordStop", class: "btn btn-danger", "Stop" }
                }
            }

            div { class: "card p-5",
                div { class: "mb-4 flex items-center justify-between gap-3",
                    div {
                        h1 { class: "text-2xl font-extrabold", "Monitor" }
                        span { id: "selectedMeta", class: "text-xs text-slate-500", "Select a device to begin" }
                    }
                    span { id: "statusChip", class: "status-pill", "Idle" }
                }
                div { id: "screenWrap", class: "screen-wrap",
                    img { id: "screen", alt: "Device screenshot" }
                    div { id: "screenEmpty", class: "empty-screen", "Select a device" }
                }
                DeviceNav {}
                p { id: "status", class: "mt-3 min-h-5 text-xs text-slate-700" }
            }

            LogsPanel {}
        }
    }
}

#[component]
fn DeviceNav() -> Element {
    rsx! {
        nav { class: "device-nav", aria_label: "Device navigation",
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
        div { class: "card mt-5 p-5",
            div { class: "mb-4 flex items-center justify-between gap-3",
                h2 { class: "text-base font-bold", "Logs" }
                span { class: "text-xs text-slate-500", "Last 300 lines" }
            }
            pre { id: "logs" }
        }
    }
}

#[component]
fn AppControls() -> Element {
    rsx! {
        aside { class: "border-l border-slate-200 bg-white/90 p-5",
            div { class: "grid gap-3",
                div { class: "field-band",
                    h3 { class: "text-xs font-bold text-slate-700", "App" }
                    input { id: "appId", placeholder: "Package or bundle id" }
                    input { id: "appPath", placeholder: "/path/to .apk or .app" }
                    div { class: "grid grid-cols-3 gap-2",
                        button { id: "install", class: "btn btn-secondary", "Install" }
                        button { id: "launch", class: "btn btn-secondary", "Launch" }
                        button { id: "terminate", class: "btn btn-secondary", "Stop" }
                    }
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
  --tw-slate-500: #64748b;
  --tw-slate-700: #334155;
  --tw-slate-900: #0f172a;
  --tw-slate-950: #020617;
  --tw-coral-50: #fff1ed;
  --tw-coral-100: #ffe0d8;
  --tw-coral-500: #f85f52;
  --tw-coral-600: #e84b40;
  --tw-white: #ffffff;
  --theme-line: rgba(226, 232, 240, .88);
  --theme-ring: rgba(248, 95, 82, .28);
  --theme-radius: 8px;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  min-height: 100vh;
  font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: var(--tw-slate-50);
  color: var(--tw-slate-950);
}
button, input, textarea { font: inherit; min-width: 0; }
button { cursor: pointer; white-space: nowrap; }
input, textarea {
  width: 100%;
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  padding: 10px 11px;
  background: var(--tw-slate-50);
  color: var(--tw-slate-950);
}
pre {
  margin: 0;
  min-height: 150px;
  max-height: 270px;
  overflow: auto;
  white-space: pre-wrap;
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  padding: 12px;
  background: var(--tw-slate-950);
  color: var(--tw-slate-100);
  font-size: 12px;
}
.min-h-screen { min-height: 100vh; }
.min-h-5 { min-height: 1.25rem; }
.grid { display: grid; }
.grid-cols-app { grid-template-columns: 260px minmax(0, 1fr) 300px; }
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
.card {
  border-radius: var(--theme-radius);
  background: var(--tw-white);
  box-shadow: 0 16px 38px rgba(15, 23, 42, .05);
  border: 1px solid var(--theme-line);
}
.btn {
  border: 1px solid transparent;
  border-radius: var(--theme-radius);
  padding: 10px 12px;
  background: var(--tw-coral-500);
  color: var(--tw-white);
  font-weight: 760;
}
.btn-secondary {
  background: var(--tw-white);
  border-color: var(--theme-line);
  color: var(--tw-slate-900);
}
.btn-danger {
  background: var(--tw-slate-950);
  border-color: var(--tw-slate-950);
  color: var(--tw-white);
}
.status-pill {
  border: 1px solid var(--tw-coral-100);
  border-radius: var(--theme-radius);
  padding: 7px 10px;
  background: var(--tw-coral-50);
  color: var(--tw-coral-600);
  font-size: 12px;
  font-weight: 750;
  white-space: nowrap;
}
.device {
  width: 100%;
  min-height: 88px;
  text-align: left;
  color: var(--tw-slate-950);
  background: var(--tw-white);
  border: 1px solid var(--theme-line);
  border-radius: var(--theme-radius);
  padding: 14px;
  display: grid;
  gap: 6px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, .04);
}
.device strong { font-size: 15px; }
.device.active {
  border-color: rgba(248, 95, 82, .58);
  outline: 3px solid var(--theme-ring);
  background: var(--tw-coral-50);
}
.muted { color: var(--tw-slate-500); font-size: 12px; overflow-wrap: anywhere; }
.screen-wrap {
  display: grid;
  place-items: center;
  min-height: 470px;
  border-radius: var(--theme-radius) var(--theme-radius) 0 0;
  background: var(--tw-slate-950);
  overflow: hidden;
  touch-action: none;
  position: relative;
}
#screen {
  max-width: 100%;
  max-height: 72vh;
  object-fit: contain;
  display: none;
  cursor: crosshair;
  user-select: none;
  -webkit-user-drag: none;
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
  background: #272b30;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  align-items: center;
  overflow: hidden;
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
  border: 1px solid rgba(226, 232, 240, .76);
  border-radius: var(--theme-radius);
  background: rgba(248, 250, 252, .86);
  padding: 14px;
  display: grid;
  gap: 10px;
}
@media (max-width: 1260px) {
  .grid-cols-app { grid-template-columns: 250px minmax(0, 1fr); }
  .border-l { grid-column: 1 / -1; border-left: 0; border-top: 1px solid var(--theme-line); }
}
@media (max-width: 920px) {
  .grid-cols-app { grid-template-columns: 1fr; }
  .border-r, .border-l { border: 0; border-top: 1px solid var(--theme-line); }
  section.bg-slate-50 { order: -1; padding: 18px; }
  .justify-end { justify-content: stretch; }
  .screen-wrap { min-height: 360px; }
}
@media (max-width: 560px) {
  .flex-wrap, .grid-cols-3 { grid-template-columns: 1fr; display: grid; }
}
"#;

const APP_SCRIPT: &str = r#"
const state = { devices: [], selected: null, ws: null, poll: null, pointerStart: null };
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
  for (const device of state.devices) {
    const btn = document.createElement('button');
    btn.className = `device ${state.selected && state.selected.id === device.id ? 'active' : ''}`;
    btn.innerHTML = `<strong>${device.name}</strong><span class="muted">${device.kind} / ${device.state}</span><span class="muted">${device.id}</span>`;
    btn.onclick = () => {
      state.selected = device;
      renderDevices();
      refreshScreenshot();
      startPolling();
    };
    el('devices').appendChild(btn);
  }
  if (!state.devices.length) el('devices').innerHTML = '<p class="muted">No running devices found</p>';
}
async function loadDevices() {
  state.devices = await json('/api/devices');
  if (state.selected && !state.devices.find(d => d.id === state.selected.id)) state.selected = null;
  renderDevices();
  setStatus(`Loaded ${state.devices.length} device(s)`);
}
async function refreshScreenshot() {
  if (!state.selected) return;
  const res = await api(`/api/devices/${selectedId()}/screenshot`);
  const blob = await res.blob();
  el('screen').src = URL.createObjectURL(blob);
  el('screen').style.display = 'block';
  el('screenEmpty').style.display = 'none';
}
function startPolling() {
  clearInterval(state.poll);
  state.poll = setInterval(() => refreshScreenshot().catch(err => setStatus(err.message)), 1000);
}
async function post(path, body) {
  await json(path, { method: 'POST', body: JSON.stringify(body || {}) });
  setStatus('Command sent');
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
    y: Math.round(y * img.naturalHeight / rect.height)
  };
}
async function sendPointerCommand(start, end) {
  const dx = Math.abs(end.x - start.x);
  const dy = Math.abs(end.y - start.y);
  if (dx < 8 && dy < 8) {
    await post(`/api/devices/${selectedId()}/input/tap`, end);
    setStatus(`Tapped ${end.x}, ${end.y}`);
  } else {
    await post(`/api/devices/${selectedId()}/input/swipe`, {
      x1: start.x,
      y1: start.y,
      x2: end.x,
      y2: end.y,
      duration_ms: 250
    });
    setStatus(`Swiped ${start.x}, ${start.y} -> ${end.x}, ${end.y}`);
  }
  refreshScreenshot().catch(err => setStatus(err.message));
}
async function sendKeyValue(key) {
  await post(`/api/devices/${selectedId()}/key`, { key });
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
  state.ws.onmessage = ev => setStatus(ev.data);
  state.ws.onopen = () => setStatus('Connected');
  state.ws.onclose = () => setStatus('WebSocket disconnected');
}
el('refresh').onclick = () => loadDevices().catch(err => setStatus(err.message));
el('shot').onclick = () => refreshScreenshot().catch(err => setStatus(err.message));
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
