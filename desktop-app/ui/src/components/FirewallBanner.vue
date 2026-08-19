<script setup lang="ts">
// bannière d'avertissement quand le pare-feu système bloque les ports UDP
// de Toolé : j'affiche les commandes à exécuter, avec copie en un clic
import { ref } from "vue";
import { useFirewallStore } from "../stores/firewall";

const firewallStore = useFirewallStore();
const copied = ref("");

async function copyCommand(cmd: string) {
  try {
    await navigator.clipboard.writeText(cmd);
    copied.value = cmd;
    setTimeout(() => {
      copied.value = "";
    }, 1500);
  } catch (e) {
    console.error("copy error:", e);
  }
}
</script>

<template>
  <div
    v-if="firewallStore.needsAction"
    class="mx-6 mt-4 px-4 py-3 rounded-xl border border-warning/50 bg-warning/10 relative z-10 flex-shrink-0"
    role="alert"
  >
    <p class="text-[12px] font-semibold text-warning">
      Pare-feu actif : Toolé ne recevra rien tant que les ports UDP
      58199 / 58200 ne sont pas autorisés.
    </p>
    <p class="text-[11px] text-on-surface-variant mt-1">
      Exécutez ces commandes dans un terminal (administrateur sous Windows) :
    </p>
    <div
      v-for="cmd in firewallStore.status?.commands ?? []"
      :key="cmd"
      class="flex items-center gap-2 mt-2"
    >
      <code
        class="flex-1 min-w-0 px-2 py-1 rounded bg-surface-container-high text-[11px] font-mono text-primary truncate"
      >{{ cmd }}</code>
      <button
        type="button"
        class="px-2 py-1 rounded text-[11px] border border-outline/50 text-on-surface-variant
               hover:text-primary hover:border-primary/50 transition-colors cursor-pointer flex-shrink-0"
        @click="copyCommand(cmd)"
      >
        {{ copied === cmd ? "Copié !" : "Copier" }}
      </button>
    </div>
  </div>
</template>