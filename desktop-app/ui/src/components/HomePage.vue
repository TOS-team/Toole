<script setup lang="ts">
// page d'accueil : j'affiche le nom de la machine et la zone de dépôt des
// fichiers à envoyer
import FileDropZone from "./FileDropZone.vue";

defineProps<{ hostname: string }>();
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col p-4 md:p-6 xl:p-8">
    <header class="mb-6 md:mb-8 flex items-center justify-between pt-4 flex-shrink-0">
      <div class="min-w-0">
        <h1
          class="text-headline-lg font-headline-lg text-on-background tracking-tight truncate"
        >
          Bienvenue
        </h1>
        <p
          class="text-label-md font-label-md text-on-surface-variant mt-1 flex items-center min-w-0"
        >
          <span class="w-2 h-2 rounded-full bg-tertiary-fixed-dim mr-2 animate-pulse shrink-0"></span>
<span class="truncate">
              Connecté en tant que
              <span class="text-primary font-semibold">{{ hostname || "inconnu" }}</span>
            </span>
        </p>
      </div>
    </header>

    <FileDropZone class="flex-1 min-h-0" />
  </div>
</template>