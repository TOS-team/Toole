// je gère les mises à jour de l'application : je vérifie sur GitHub si une
// nouvelle version existe, je télécharge le binaire signé et je relance l'app
// après installation. L'état est consultable par l'interface (paramètres).
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { check, type DownloadEvent } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateStatus =
  | "idle" // je n'ai encore rien vérifié
  | "checking" // je suis en train d'interroger GitHub
  | "up-to-date" // pas de mise à jour disponible
  | "available" // une nouvelle version est prête à être installée
  | "downloading" // je télécharge le paquet
  | "installing" // j'installe puis je relance
  | "error"; // la vérification ou l'installation a échoué

export const useUpdaterStore = defineStore("updater", () => {
  const status = ref<UpdateStatus>("idle");
  const error = ref("");
  const newVersion = ref("");
  const currentVersion = ref("");
  const notes = ref("");
  const downloaded = ref(0);
  const contentLength = ref(0);

  // petite aide : mes états "occupé" pour griser le bouton d'installation
  const busy = computed(
    () =>
      status.value === "checking" ||
      status.value === "downloading" ||
      status.value === "installing",
  );

  function progressPercent(): number {
    if (contentLength.value <= 0) return 0;
    return Math.min(
      100,
      Math.round((downloaded.value / contentLength.value) * 100),
    );
  }

  // je demande à GitHub s'il existe une version plus récente. Si `asap` est
  // faux (check silencieux au démarrage), je ne remonte pas l'état "à jour".
  async function checkForUpdate(asap = true): Promise<boolean> {
    if (busy.value) return false;
    status.value = "checking";
    error.value = "";
    try {
      const update = await check();
      if (update) {
        currentVersion.value = update.currentVersion;
        newVersion.value = update.version;
        notes.value = update.body ?? "";
        status.value = "available";
        return true;
      }
      status.value = asap ? "up-to-date" : "idle";
      return false;
    } catch (e) {
      // si le check est silencieux (démarrage, panneau ouvert d'office), je ne
      // remonte jamais d'erreur : un utilisateur hors-ligne ne doit rien voir.
      if (!asap) {
        status.value = "idle";
        error.value = "";
        return false;
      }
      // check explicite : je choisis un message lisible selon la cause
      const msg = String(e).toLowerCase();
      if (msg.includes("json")) {
        error.value = "Aucune version publiée pour le moment.";
      } else if (
        msg.includes("network") ||
        msg.includes("fetch") ||
        msg.includes("reqwest") ||
        msg.includes("timed out") ||
        msg.includes("connection")
      ) {
        error.value = "Vérifie ta connexion internet puis réessaie.";
      } else {
        error.value = String(e);
      }
      status.value = "error";
      return false;
    }
  }

  // je télécharge et j'installe la nouvelle version, puis je relance l'app
  async function install(): Promise<void> {
    if (status.value !== "available") return;
    status.value = "downloading";
    error.value = "";
    downloaded.value = 0;
    contentLength.value = 0;

    const update = await check(); // je récupère la mise à jour annoncée
    if (!update) {
      status.value = "up-to-date";
      return;
    }

    try {
      await update.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case "Started":
            contentLength.value = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloaded.value += event.data.chunkLength;
            break;
          case "Finished":
            status.value = "installing";
            break;
        }
      });
      // relance l'application avec la version fraîche
      await relaunch();
    } catch (e) {
      status.value = "error";
      error.value = String(e);
    }
  }

  return {
    status,
    error,
    newVersion,
    currentVersion,
    notes,
    downloaded,
    contentLength,
    busy,
    progressPercent,
    checkForUpdate,
    install,
  };
});