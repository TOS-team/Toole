const { invoke } = window.__TAURI__?.core ?? {};
const { listen } = window.__TAURI__?.event ?? {};

const peerList = document.getElementById("peer-list");
const peerEmptyState = document.getElementById("no-peers");
const selectAllPeers = document.getElementById("select-all-peers");
const logContent = document.getElementById("log-content");
const btnStart = document.getElementById("btn-start");
const btnStop = document.getElementById("btn-stop");
const btnAbout = document.getElementById("btn-about");
const aboutModal = document.getElementById("about-modal");
const aboutClose = document.getElementById("about-close");

function log(msg) {
  if (!logContent) return;
  const line = document.createElement("div");
  line.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  logContent.appendChild(line);
}

if (btnStart) {
  btnStart.addEventListener("click", async () => {
    log("Démarrage...");
    try {
      await invoke("start_discovery");
      log("Discovery démarré");
    } catch (e) {
      log(`Erreur : ${e}`);
    }
  });
}

if (btnStop) {
  btnStop.addEventListener("click", async () => {
    log("Arrêt...");
    try {
      await invoke("stop_discovery");
      log("Discovery arrêté");
    } catch (e) {
      log(`Erreur : ${e}`);
    }
  });
}

function openAbout() {
  if (!aboutModal) return;
  aboutModal.classList.add("is-open");
}

function closeAbout() {
  if (!aboutModal) return;
  aboutModal.classList.remove("is-open");
}

if (btnAbout) {
  btnAbout.addEventListener("click", openAbout);
}

if (aboutClose) {
  aboutClose.addEventListener("click", closeAbout);
}

if (aboutModal) {
  aboutModal.addEventListener("click", (event) => {
    if (event.target === aboutModal) closeAbout();
  });
}

window.addEventListener("keydown", (event) => {
  if (event.key === "Escape") closeAbout();
});

function makePeerCard(hostname, addr) {
  const li = document.createElement("li");
  li.className = "peer-item";
  li.dataset.hostname = hostname;
  li.tabIndex = 0;
  li.setAttribute("role", "button");
  li.setAttribute("aria-selected", "false");

  const avatar = document.createElement("div");
  avatar.className = "peer-avatar";
  avatar.textContent = hostname.trim().charAt(0).toUpperCase() || "?";

  const meta = document.createElement("div");
  meta.className = "peer-meta";

  const name = document.createElement("div");
  name.className = "peer-name";
  name.textContent = hostname;

  const ip = document.createElement("div");
  ip.className = "peer-ip";
  ip.textContent = addr;

  const check = document.createElement("div");
  check.className = "peer-check";
  check.textContent = "✓";

  meta.append(name, ip);
  li.append(avatar, meta, check);
  return li;
}

function getPeerCards() {
  if (!peerList) return [];
  return Array.from(peerList.querySelectorAll(".peer-item"));
}

function findPeerCard(hostname) {
  if (!peerList) return null;
  const safeHostname = (window.CSS && CSS.escape) ? CSS.escape(hostname) : hostname.replace(/"/g, '\\"');
  return peerList.querySelector(`[data-hostname="${safeHostname}"]`);
}

function setPeerSelected(card, selected) {
  if (!card) return;
  card.classList.toggle("selected", selected);
  card.setAttribute("aria-selected", selected ? "true" : "false");
  const check = card.querySelector(".peer-check");
  if (check) check.setAttribute("aria-hidden", selected ? "false" : "true");
}

function syncSelectAllState() {
  if (!selectAllPeers) return;
  const cards = getPeerCards();
  if (!cards.length) {
    selectAllPeers.checked = false;
    selectAllPeers.indeterminate = false;
    return;
  }
  const selectedCount = cards.filter((card) => card.classList.contains("selected")).length;
  selectAllPeers.checked = selectedCount === cards.length;
  selectAllPeers.indeterminate = selectedCount > 0 && selectedCount < cards.length;
}

function selectAllPeerCards(selected) {
  getPeerCards().forEach((card) => setPeerSelected(card, selected));
  syncSelectAllState();
}

function updatePeerEmptyState() {
  if (!peerList || !peerEmptyState) return;
  const hasPeers = peerList.querySelectorAll(".peer-item").length > 0;
  if (hasPeers) {
    peerEmptyState.remove();
  } else if (!peerList.contains(peerEmptyState)) {
    peerList.appendChild(peerEmptyState);
  }
}

if (peerList) {
  peerList.addEventListener("click", (event) => {
    const card = event.target.closest(".peer-item");
    if (!card || !peerList.contains(card)) return;
    setPeerSelected(card, !card.classList.contains("selected"));
    syncSelectAllState();
  });

  peerList.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    const card = event.target.closest(".peer-item");
    if (!card || !peerList.contains(card)) return;
    event.preventDefault();
    setPeerSelected(card, !card.classList.contains("selected"));
    syncSelectAllState();
  });
}

if (selectAllPeers) {
  selectAllPeers.addEventListener("change", () => {
    selectAllPeerCards(selectAllPeers.checked);
  });
}

if (listen) {
  listen("log", (event) => log(event.payload));
  listen("peer-found", (event) => {
    const { hostname, addr } = event.payload;
    if (!peerList) return;
    const existing = findPeerCard(hostname);
    if (existing) existing.remove();
    if (peerEmptyState && peerEmptyState.parentElement === peerList) {
      peerEmptyState.remove();
    }
    const card = makePeerCard(hostname, addr);
    peerList.appendChild(card);
    if (selectAllPeers?.checked) {
      setPeerSelected(card, true);
    }
    updatePeerEmptyState();
    syncSelectAllState();
    log(`Pair trouvé : ${hostname}`);
  });
  listen("peer-lost", (event) => {
    const { hostname } = event.payload;
    if (!peerList) return;
    const item = findPeerCard(hostname);
    if (item) item.remove();
    updatePeerEmptyState();
    syncSelectAllState();
    log(`Pair perdu : ${hostname}`);
  });
}

updatePeerEmptyState();
syncSelectAllState();

if (invoke) {
  invoke("start_discovery").catch(console.error);
}
