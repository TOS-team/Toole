<script setup lang="ts">
// j'affiche une icône SVG en chargeant son fichier brut depuis assets/icons
import { computed } from "vue";

const props = defineProps<{ name: string; size?: number }>();

// je précharge toutes les icônes au build pour les servir par nom
const icons = import.meta.glob("../assets/icons/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const svg = computed(
  () => icons[`../assets/icons/${props.name}.svg`] ?? "",
);
</script>

<template>
  <span
    class="inline-flex items-center justify-center leading-none shrink-0"
    :style="{ fontSize: (size ?? 24) + 'px' }"
    aria-hidden="true"
    v-html="svg"
  />
</template>