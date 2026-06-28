// je prepare la fonction pour appeler Tauri depuis le frontend
const _invoke = (cmd, args) => window.__TAURI_INTERNALS__.invoke(cmd, args);

// je recupere les elements du DOM
const peerList = document.getElementById("peer-list");
const peerEmptyState = document.getElementById("no-peers");
const selectAllPeers = document.getElementById("select-all-peers");
const btnAbout = document.getElementById("btn-about");
const welcomeHostname = document.getElementById("welcome-hostname");
const aboutModal = document.getElementById("about-modal");
const aboutClose = document.getElementById("about-close");

// gestion de la modale a propos
function openAbout() {
  if (!aboutModal) return;
  aboutModal.classList.add("is-open");
}

// je ferme la modale a propos
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

// je cree un element HTML pour representer un pair dans la liste
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

// je recupere tous les elements de la liste des pairs
function getPeerCards() {
  if (!peerList) return [];
  return Array.from(peerList.querySelectorAll(".peer-item"));
}

// je cherche une carte de pair par son hostname
function findPeerCard(hostname) {
  if (!peerList) return null;
  const safeHostname = (window.CSS && CSS.escape) ? CSS.escape(hostname) : hostname.replace(/"/g, '\\"');
  return peerList.querySelector(`[data-hostname="${safeHostname}"]`);
}

// je marque un pair comme selectionne ou non
function setPeerSelected(card, selected) {
  if (!card) return;
  card.classList.toggle("selected", selected);
  card.setAttribute("aria-selected", selected ? "true" : "false");
  const check = card.querySelector(".peer-check");
  if (check) check.setAttribute("aria-hidden", selected ? "false" : "true");
}

// je synchronise l'etat de la case "tout selectionner"
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

// je selectionne ou deselectionne tous les pairs
function selectAllPeerCards(selected) {
  getPeerCards().forEach((card) => setPeerSelected(card, selected));
  syncSelectAllState();
}

// j'affiche ou masque le message "aucun appareil" selon la liste
function updatePeerEmptyState() {
  if (!peerList || !peerEmptyState) return;
  const hasPeers = peerList.querySelectorAll(".peer-item").length > 0;
  if (hasPeers) {
    peerEmptyState.remove();
  } else if (!peerList.contains(peerEmptyState)) {
    peerList.appendChild(peerEmptyState);
  }
}

// gestion des clics et du clavier sur la liste des pairs
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

// quand l'utilisateur coche "tout selectionner"
if (selectAllPeers) {
  selectAllPeers.addEventListener("change", () => {
    selectAllPeerCards(selectAllPeers.checked);
  });
}

// je synchronise la liste des pairs avec le backend
async function refreshPeers() {
  if (!peerList) return;
  try {
    const peers = await _invoke("get_peers");
    // je garde les hostnames actuels pour savoir quoi ajouter/supprimer
    const currentHostnames = new Set(getPeerCards().map(c => c.dataset.hostname));
    const newHostnames = new Set(peers.map(p => p.hostname));

    // je supprime les pairs qui ne sont plus la
    for (const h of currentHostnames) {
      if (!newHostnames.has(h)) {
        const card = findPeerCard(h);
        if (card) card.remove();
      }
    }

    // j'ajoute les nouveaux pairs
    for (const p of peers) {
      if (!currentHostnames.has(p.hostname)) {
        if (peerEmptyState && peerEmptyState.parentElement === peerList) {
          peerEmptyState.remove();
        }
        const card = makePeerCard(p.hostname, p.addr);
        peerList.appendChild(card);
      }
    }

    updatePeerEmptyState();
    syncSelectAllState();
  } catch (e) {
    console.error("Refresh peers error:", e);
  }
}

// j'initialise l'etat de la liste
updatePeerEmptyState();
syncSelectAllState();

// au chargement je recupere le hostname et je lance le discovery automatiquement
(async function init() {
  try {
    const hostname = await _invoke("get_hostname");
    console.log("Hostname:", hostname);
    if (welcomeHostname) welcomeHostname.textContent = hostname || "inconnu";
    // je lance la decouverte automatique au demarrage
    await _invoke("start_discovery");
    // je commence le polling des pairs toutes les 2 secondes
    setInterval(refreshPeers, 2000);
  } catch (e) {
    console.error("Init error:", e);
  }
})();

// quand l'utilisateur ferme la fenetre, j'arrete le discovery
window.addEventListener("beforeunload", async () => {
  try {
    await _invoke("stop_discovery");
  } catch (e) {
    console.error(e);
  }
});
