<script setup lang="ts">
// bannière d'avertissement quand le pare-feu système bloque les ports UDP
// de Toolé : j'affiche les commandes à exécuter, avec copie en un clic
import { openUrl } from "@tauri-apps/plugin-opener";
import { useFirewallStore } from "../stores/firewall";
import FirewallCommands from "./FirewallCommands.vue";

const firewallStore = useFirewallStore();

// le guide de dépannage du site officiel, section pare-feu
const DEPANNAGE_URL = "https://toole-site.web.app/guide/index.html#/depannage";
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
    <FirewallCommands :commands="firewallStore.status?.commands ?? []" />
    <p class="text-[11px] text-on-surface-variant/70 mt-2">
      Toujours bloqué après ces commandes ?
      <a
        href="#"
        class="underline decoration-warning/60 underline-offset-2 hover:text-warning transition-colors"
        @click.prevent="openUrl(DEPANNAGE_URL)"
      >Consultez la section Dépannage du site.</a>
    </p>
  </div>
</template>