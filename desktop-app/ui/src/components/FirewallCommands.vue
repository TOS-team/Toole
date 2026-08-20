<script setup lang="ts">
// liste des commandes pare-feu à exécuter, avec copie en un clic — partagée
// entre la bannière de la sidebar et la page Paramètres
import { ref } from "vue";

defineProps<{ commands: string[] }>();

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
    v-for="cmd in commands"
    :key="cmd"
    class="flex items-center gap-2 mt-2"
  >
    <code
      class="flex-1 min-w-0 px-2 py-1 rounded bg-surface-container-lowest text-[11px] font-mono text-primary truncate"
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
</template>