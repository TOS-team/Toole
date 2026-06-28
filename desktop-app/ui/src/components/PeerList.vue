<script setup lang="ts">
import { computed } from "vue";
import { usePeersStore } from "../stores/peers";

const peersStore = usePeersStore();

const allSelected = computed(() => {
  const p = peersStore.peers;
  return p.length > 0 && peersStore.selectedHostnames.size === p.length;
});

const indeterminate = computed(() => {
  const s = peersStore.selectedHostnames.size;
  return s > 0 && s < peersStore.peers.length;
});

function onSelectAllChange(e: Event) {
  peersStore.selectAll((e.target as HTMLInputElement).checked);
}

function peerKey(hostname: string, addr: string) {
  return hostname + "@" + addr;
}
</script>

<template>
  <div class="panel flex-1 flex flex-col min-h-[140px] rounded-xl p-3.5 gap-2">
    <div class="flex items-center justify-between">
      <span class="panel-label text-white/60">Appareils disponibles</span>
      <label
        v-if="peersStore.peers.length"
        class="flex items-center gap-1.5 text-[11px] text-dim cursor-pointer select-none"
      >
        <input
          type="checkbox"
          class="w-3 h-3 accent-red-brand cursor-pointer"
          :checked="allSelected"
          :indeterminate="indeterminate"
          @change="onSelectAllChange"
        />
      </label>
    </div>

    <ul
      v-if="peersStore.peers.length"
      class="list-none flex flex-col gap-1.5 flex-1 min-h-0 overflow-y-auto pr-0.5 scrollbar-gutter-stable"
    >
      <li
        v-for="p in peersStore.peers"
        :key="peerKey(p.hostname, p.addr)"
        class="flex items-center gap-2.5 px-2.5 py-2.5 text-[13px]
               bg-white/4 border border-white/7 rounded-[10px]
               cursor-pointer transition-all duration-150
               hover:bg-red-soft hover:border-red-brand/30 hover:-translate-y-px
               focus-visible:outline-2 focus-visible:outline-red-brand focus-visible:outline-offset-2"
        :class="{
          'bg-red-brand/22 border-red-brand/70 shadow-[0_0_0_1px_rgba(232,40,43,0.28),0_8px_18px_rgba(0,0,0,0.22)]':
            peersStore.selectedHostnames.has(p.hostname),
        }"
        tabindex="0"
        role="button"
        :aria-selected="peersStore.selectedHostnames.has(p.hostname)"
        @click="peersStore.toggleSelection(p.hostname)"
        @keydown.enter.prevent="peersStore.toggleSelection(p.hostname)"
        @keydown.space.prevent="peersStore.toggleSelection(p.hostname)"
      >
        <div
          class="w-7 h-7 rounded-[7px] flex items-center justify-center text-[13px] shrink-0
                 bg-red-soft border border-red-brand/25"
        >
          {{ (p.hostname.trim().charAt(0) || "?").toUpperCase() }}
        </div>
        <div class="min-w-0 flex-1 flex flex-col gap-0.5">
          <div class="truncate font-medium">{{ p.hostname }}</div>
          <div class="text-[11px] text-muted font-mono shrink-0">{{ p.addr }}</div>
        </div>
        <div
          class="w-[18px] h-[18px] rounded-full flex items-center justify-center text-[11px] font-bold
                 text-white shrink-0 transition-all duration-150"
          :class="
            peersStore.selectedHostnames.has(p.hostname)
              ? 'bg-red-brand/95 opacity-100 scale-100'
              : 'bg-red-brand/20 border border-red-brand/45 opacity-25 scale-90'
          "
        >
          ✓
        </div>
      </li>
    </ul>

    <div
      v-else
      class="flex-1 flex items-center justify-center text-[13px] text-muted font-light"
    >
      Aucun appareil détecté
    </div>
  </div>
</template>
