const { invoke } = window.__TAURI__?.core ?? {};
const { listen } = window.__TAURI__?.event ?? {};

const peerList = document.getElementById("peer-list");
const logContent = document.getElementById("log-content");
const btnStart = document.getElementById("btn-start");
const btnStop = document.getElementById("btn-stop");

function log(msg) {
  const line = document.createElement("div");
  line.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  logContent.appendChild(line);
}

btnStart.addEventListener("click", async () => {
  log("Démarrage...");
  try {
    await invoke("start_discovery");
    log("Discovery démarré");
  } catch (e) {
    log(`Erreur : ${e}`);
  }
});

btnStop.addEventListener("click", async () => {
  log("Arrêt...");
  try {
    await invoke("stop_discovery");
    log("Discovery arrêté");
  } catch (e) {
    log(`Erreur : ${e}`);
  }
});

if (listen) {
  listen("log", (event) => log(event.payload));
  listen("peer-found", (event) => {
    const { hostname, addr } = event.payload;
    const li = document.createElement("li");
    li.textContent = `${hostname} (${addr})`;
    peerList.appendChild(li);
    log(`Pair trouvé : ${hostname}`);
  });
  listen("peer-lost", (event) => {
    const { hostname } = event.payload;
    const items = peerList.querySelectorAll("li");
    for (const item of items) {
      if (item.textContent.startsWith(hostname)) {
        item.remove();
        break;
      }
    }
    log(`Pair perdu : ${hostname}`);
  });
}
