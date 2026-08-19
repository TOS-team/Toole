<script setup lang="ts">
// liste des appareils détectés, avec sélection multiple, bouton d'actualisation
// et ajout manuel d'un appareil par adresse IP (quand la découverte est
// bloquée). Le mode manuel est replié par défaut : l'utilisateur doit
// l'activer pour l'utiliser, et il peut retirer les appareils ainsi ajoutés.
import { ref } from "vue";
import { usePeersStore } from "../stores/peers";
import { invoke } from "../tauri";
import Icon from "./Icon.vue";

const peersStore = usePeersStore();
const spinning = ref(false);
const manualOpen = ref(false);
const manualIp = ref("");
const manualError = ref("");

// je marque les appareils ajoutés par IP pour leur proposer un retrait
function isManual(id: string) {
  return id.startsWith("manual-");
}

function peerKey(id: string, addr: string) {
  return id + "@" + addr;
}

// je relance la découverte (arrêt + départ) puis le polling pour rafraîchir
// immédiatement la liste
async function onRefresh() {
  spinning.value = true;
  try {
    await invoke("stop_discovery");
    await invoke("start_discovery");
  } catch (e) {
    console.error("restart discovery error:", e);
  }
  peersStore.startPolling();
  setTimeout(() => {
    spinning.value = false;
  }, 800);
}

// j'ajoute un appareil par IP : le backend valide l'adresse (IPv4 privé)
async function addManual() {
  manualError.value = "";
  try {
    await invoke("add_peer", { ip: manualIp.value });
    manualIp.value = "";
  } catch (e) {
    manualError.value = String(e);
  }
}

// je retire un appareil ajouté manuellement (backend + liste locale, sans
// attendre le polling)
async function removeManual(id: string) {
  try {
    await invoke("remove_peer", { id });
    peersStore.removePeer(id);
  } catch (e) {
    console.error("remove peer error:", e);
  }
}
</script>

<template>
  <div class="flex flex-col min-h-0 px-6 pt-3 pb-6 gap-3">
    <div class="flex justify-between items-center relative z-10 flex-shrink-0">
      <h3 class="text-label-sm font-label-sm text-on-surface-variant uppercase">
        Appareils ({{ peersStore.peers.length }})
      </h3>
      <button
        type="button"
        title="Rafraîchir"
        class="text-on-surface-variant hover:text-primary transition-colors cursor-pointer"
        @click="onRefresh"
      >
        <Icon
          name="sync"
          :size="16"
          class="transition-transform"
          :class="spinning ? 'animate-spin' : ''"
        />
      </button>
    </div>

    <ul
      v-if="peersStore.peers.length"
      class="flex flex-col gap-2 flex-1 min-h-0 overflow-y-auto pr-0.5 relative z-10"
    >
      <li
        v-for="p in peersStore.peers"
        :key="peerKey(p.id, p.addr)"
        class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-[13px]
               bg-surface-container-high border border-outline/50
               cursor-pointer transition-all duration-150
               hover:border-primary/50 hover:-translate-y-px
               focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-2"
        :class="{
          'bg-primary/15 border-primary/70': peersStore.selectedIds.has(p.id),
        }"
        tabindex="0"
        role="button"
        :aria-selected="peersStore.selectedIds.has(p.id)"
        @click="peersStore.toggleSelection(p.id)"
        @keydown.enter.prevent="peersStore.toggleSelection(p.id)"
        @keydown.space.prevent="peersStore.toggleSelection(p.id)"
      >
        <div
          class="w-8 h-8 rounded-lg border border-outline bg-surface-variant flex items-center justify-center text-on-surface text-[13px] font-bold shrink-0"
        >
          {{ (p.id.trim().charAt(0) || "?").toUpperCase() }}
        </div>
        <div class="min-w-0 flex-1 flex flex-col gap-0.5">
          <div class="truncate text-on-surface font-medium">{{ p.id }}</div>
          <div class="text-[11px] text-on-surface-variant font-mono truncate">{{ p.addr }}</div>
        </div>
        <span
          v-if="isManual(p.id)"
          class="text-[9px] uppercase tracking-wide px-1.5 py-0.5 rounded border border-outline/60 text-on-surface-variant/80 shrink-0"
          title="Appareil ajouté manuellement"
        >manuel</span>
        <button
          v-if="isManual(p.id)"
          type="button"
          title="Retirer cet appareil"
          aria-label="Retirer {{ p.id }}"
          class="p-1 rounded-md text-on-surface-variant/60 hover:text-error hover:bg-error/10 transition-colors cursor-pointer shrink-0"
          @click.stop="removeManual(p.id)"
        >
          <Icon name="delete" :size="14" />
        </button>
        <span
          class="w-2 h-2 rounded-full shrink-0 transition-colors duration-150"
          :class="
            peersStore.selectedIds.has(p.id)
              ? 'bg-primary'
              : 'bg-outline'
          "
        ></span>
      </li>
    </ul>

    <div
      v-else
      class="flex-1 rounded-xl border border-dashed border-outline-variant flex flex-col items-center justify-center text-center p-4 bg-surface-container/30 relative z-10"
    >
      <Icon name="phonelink-off" :size="28" class="text-on-surface-variant opacity-50 mb-3" />
      <p class="text-label-md font-label-md text-on-surface-variant">
        Aucun appareil détecté
      </p>
      <p class="text-[11px] text-on-surface-variant/70 mt-1 max-w-[200px]">
        Vérifiez que les machines sont sur le même réseau, que le pare-feu
        autorise Toolé et que l'isolation client du routeur est désactivée.
      </p>
      <p class="text-[11px] text-on-surface-variant/70 mt-1 max-w-[200px]">
        Vous pouvez aussi ajouter un appareil par adresse IP
        (bouton « Ajouter un appareil par IP » en bas).
      </p>
      <p v-if="peersStore.discoveryError" class="text-[11px] text-error mt-2 max-w-[200px]">
        Erreur découverte : {{ peersStore.discoveryError }}
      </p>
    </div>

    <button
      v-if="!manualOpen"
      type="button"
      class="flex items-center justify-center gap-1.5 relative z-10 flex-shrink-0
             w-full px-3 py-2 rounded-lg border border-dashed border-outline-variant
             text-[11px] text-on-surface-variant hover:text-primary hover:border-primary/50
             transition-colors cursor-pointer"
      @click="manualOpen = true"
    >
      <Icon name="add" :size="13" />
      Ajouter un appareil par IP
    </button>

    <form
      v-else
      class="flex gap-2 relative z-10 flex-shrink-0"
      @submit.prevent="addManual"
    >
      <input
        v-model="manualIp"
        type="text"
        inputmode="decimal"
        autocomplete="off"
        spellcheck="false"
        placeholder="Adresse IP (ex. 192.168.1.42)"
        aria-label="Adresse IP de l'appareil"
        class="flex-1 min-w-0 px-3 py-2 rounded-lg text-[12px] font-mono
               bg-surface-container-high border border-outline/50
               text-on-surface placeholder:text-on-surface-variant/50
               focus-visible:outline-2 focus-visible:outline-primary"
      />
      <button
        type="submit"
        title="Ajouter l'appareil"
        :disabled="!manualIp.trim()"
        class="px-3 rounded-lg border border-outline/50 text-on-surface-variant
               hover:text-primary hover:border-primary/50 transition-colors
               disabled:opacity-40 disabled:pointer-events-none cursor-pointer"
      >
        <Icon name="add" :size="16" />
      </button>
      <button
        type="button"
        title="Replier"
        aria-label="Replier"
        class="px-2 rounded-lg border border-outline/50 text-on-surface-variant
               hover:text-on-surface transition-colors cursor-pointer"
        @click="manualOpen = false"
      >
        <Icon name="close" :size="14" />
      </button>
    </form>
    <p v-if="manualError" class="text-[11px] text-error flex-shrink-0" role="alert">
      {{ manualError }}
    </p>
  </div>
</template>