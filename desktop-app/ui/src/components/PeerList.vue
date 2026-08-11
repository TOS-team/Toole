<script setup lang="ts">
import { ref } from "vue";
import { usePeersStore } from "../stores/peers";
import { invoke } from "../tauri";
import Icon from "./Icon.vue";

const peersStore = usePeersStore();
const spinning = ref(false);

function peerKey(id: string, addr: string) {
  return id + "@" + addr;
}

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
      <p class="text-[11px] text-on-surface-variant/70 mt-1 max-w-[180px]">
        Ouvrez l'application sur une autre machine du réseau.
      </p>
      <p v-if="peersStore.discoveryError" class="text-[11px] text-error mt-2 max-w-[200px]">
        Erreur découverte : {{ peersStore.discoveryError }}
      </p>
    </div>
  </div>
</template>