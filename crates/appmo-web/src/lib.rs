pub fn dashboard_html() -> &'static str {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Appmo</title>
  <style>
    :root {
      color-scheme: light dark;
      --bg: #f7f7f4;
      --panel: #ffffff;
      --text: #1c1c1a;
      --muted: #686b65;
      --line: #dddfd8;
      --accent: #0f766e;
      --accent-strong: #115e59;
      --danger: #b42318;
      --code: #111827;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: var(--bg);
      color: var(--text);
    }
    header {
      display: flex;
      gap: 16px;
      align-items: center;
      justify-content: space-between;
      padding: 16px 20px;
      border-bottom: 1px solid var(--line);
      background: rgba(255,255,255,.84);
      position: sticky;
      top: 0;
      z-index: 2;
      backdrop-filter: blur(16px);
    }
    h1 { margin: 0; font-size: 20px; font-weight: 700; }
    h2 { margin: 0 0 12px; font-size: 15px; }
    main {
      display: grid;
      grid-template-columns: 300px minmax(0, 1fr) 340px;
      gap: 16px;
      padding: 16px;
      min-height: calc(100vh - 66px);
    }
    section, aside {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 14px;
      min-width: 0;
    }
    .actions {
      display: flex;
      gap: 8px;
      align-items: center;
    }
    input, select, button, textarea {
      font: inherit;
      border: 1px solid var(--line);
      border-radius: 6px;
      padding: 9px 10px;
      background: #fff;
      color: var(--text);
      min-width: 0;
    }
    input, textarea { width: 100%; }
    button {
      cursor: pointer;
      background: var(--accent);
      border-color: var(--accent);
      color: white;
      font-weight: 650;
      white-space: nowrap;
    }
    button.secondary { background: #fff; color: var(--text); border-color: var(--line); }
    button.danger { background: var(--danger); border-color: var(--danger); }
    button:disabled { opacity: .55; cursor: not-allowed; }
    .device-list { display: grid; gap: 8px; }
    .device {
      width: 100%;
      text-align: left;
      color: var(--text);
      background: #fff;
      border-color: var(--line);
      display: grid;
      gap: 4px;
    }
    .device.active { outline: 2px solid var(--accent); }
    .muted { color: var(--muted); font-size: 13px; }
    .screen-wrap {
      display: grid;
      place-items: center;
      min-height: 480px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: #202124;
      overflow: hidden;
      touch-action: none;
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
    .empty-screen { color: #d7d9d2; }
    .grid { display: grid; gap: 10px; }
    .row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    .row3 { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; }
    .toolbar { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; }
    pre {
      margin: 0;
      min-height: 180px;
      max-height: 360px;
      overflow: auto;
      white-space: pre-wrap;
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 10px;
      background: var(--code);
      color: #e8ece7;
      font-size: 12px;
    }
    .status {
      min-height: 22px;
      color: var(--muted);
      font-size: 13px;
      overflow-wrap: anywhere;
    }
    @media (max-width: 1100px) {
      main { grid-template-columns: 260px minmax(0, 1fr); }
      aside.controls { grid-column: 1 / -1; }
    }
    @media (max-width: 760px) {
      header { align-items: stretch; flex-direction: column; }
      main { grid-template-columns: 1fr; }
      .screen-wrap { min-height: 320px; }
    }
  </style>
</head>
<body>
  <header>
    <h1>Appmo</h1>
    <div class="actions">
      <button id="refresh" class="secondary">Refresh</button>
    </div>
  </header>
  <main>
    <aside>
      <h2>Devices</h2>
      <div id="devices" class="device-list"></div>
    </aside>
    <section>
      <div class="toolbar">
        <button id="shot" class="secondary">Screenshot</button>
        <button id="logsBtn" class="secondary">Logs</button>
        <button id="recordStart" class="secondary">Start recording</button>
        <button id="recordStop" class="danger">Stop recording</button>
      </div>
      <div class="screen-wrap" id="screenWrap">
        <img id="screen" alt="Device screenshot">
        <div class="empty-screen" id="screenEmpty">Select a device</div>
      </div>
      <p class="status" id="status"></p>
      <pre id="logs"></pre>
    </section>
    <aside class="controls">
      <h2>Control</h2>
      <div class="grid">
        <div class="row">
          <input id="tapX" type="number" min="0" placeholder="Tap X">
          <input id="tapY" type="number" min="0" placeholder="Tap Y">
        </div>
        <button id="tap">Tap</button>
        <div class="row">
          <input id="swipeX1" type="number" min="0" placeholder="X1">
          <input id="swipeY1" type="number" min="0" placeholder="Y1">
        </div>
        <div class="row">
          <input id="swipeX2" type="number" min="0" placeholder="X2">
          <input id="swipeY2" type="number" min="0" placeholder="Y2">
        </div>
        <button id="swipe">Swipe</button>
        <input id="text" placeholder="Text input">
        <button id="sendText">Type text</button>
        <input id="key" placeholder="Key event, e.g. ENTER or 66">
        <button id="sendKey">Send key</button>
        <h2>App</h2>
        <input id="appId" placeholder="Package or bundle id">
        <input id="appPath" placeholder="/path/to .apk or .app">
        <div class="row3">
          <button id="install" class="secondary">Install</button>
          <button id="launch" class="secondary">Launch</button>
          <button id="terminate" class="secondary">Stop</button>
        </div>
      </div>
    </aside>
  </main>
  <script>
    const state = { devices: [], selected: null, ws: null, poll: null, pointerStart: null };
    const el = id => document.getElementById(id);

    function setStatus(text) { el('status').textContent = text || ''; }
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
      for (const device of state.devices) {
        const btn = document.createElement('button');
        btn.className = `device ${state.selected && state.selected.id === device.id ? 'active' : ''}`;
        btn.innerHTML = `<strong>${device.name}</strong><span class="muted">${device.kind} · ${device.state}</span><span class="muted">${device.id}</span>`;
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
      const img = el('screen');
      if (!img.naturalWidth || !img.naturalHeight) throw new Error('Screenshot is not ready');
      const rect = img.getBoundingClientRect();
      const x = Math.max(0, Math.min(rect.width, event.clientX - rect.left));
      const y = Math.max(0, Math.min(rect.height, event.clientY - rect.top));
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
    function n(id) { return Number(el(id).value || 0); }
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
    el('tap').onclick = () => post(`/api/devices/${selectedId()}/input/tap`, { x: n('tapX'), y: n('tapY') }).catch(err => setStatus(err.message));
    el('swipe').onclick = () => post(`/api/devices/${selectedId()}/input/swipe`, { x1: n('swipeX1'), y1: n('swipeY1'), x2: n('swipeX2'), y2: n('swipeY2'), duration_ms: 300 }).catch(err => setStatus(err.message));
    el('sendText').onclick = () => post(`/api/devices/${selectedId()}/input/text`, { text: el('text').value }).catch(err => setStatus(err.message));
    el('sendKey').onclick = () => post(`/api/devices/${selectedId()}/key`, { key: el('key').value }).catch(err => setStatus(err.message));
    el('install').onclick = () => post(`/api/devices/${selectedId()}/app/install`, { path: el('appPath').value }).catch(err => setStatus(err.message));
    el('launch').onclick = () => post(`/api/devices/${selectedId()}/app/launch`, { app_id: el('appId').value }).catch(err => setStatus(err.message));
    el('terminate').onclick = () => post(`/api/devices/${selectedId()}/app/terminate`, { app_id: el('appId').value }).catch(err => setStatus(err.message));
    el('recordStart').onclick = () => post(`/api/devices/${selectedId()}/record/start`, {}).catch(err => setStatus(err.message));
    el('recordStop').onclick = () => post(`/api/devices/${selectedId()}/record/stop`, {}).catch(err => setStatus(err.message));
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
    connectWs();
    loadDevices().catch(err => setStatus(err.message));
  </script>
</body>
</html>"#
}
