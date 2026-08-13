<script setup lang="ts">
// page des transferts : je montre les transferts en attente de validation
// (envois et demandes entrantes) et ceux en cours (les finis sont dans
// l'historique)
import { computed } from "vue";
import { useTransfersStore } from "../stores/transfers";
import TransferList from "./TransferList.vue";
import Icon from "./Icon.vue";

const store = useTransfersStore();
const active = computed(() =>
  store.transfers.filter(
    (t) =>
      t.status === "pending" || t.status === "incoming" || t.status === "running",
  ),
);
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col p-4 md:p-6 xl:p-8">
    <header class="mb-6 md:mb-8 flex items-center justify-between pt-4 flex-shrink-0">
      <div class="min-w-0">
          <h1
            class="text-headline-lg font-headline-lg text-on-background tracking-tight truncate"
          >
            Transfert
          </h1>
          <p class="text-label-md font-label-md text-on-surface-variant mt-1">
            {{ active.length }} transfert{{ active.length > 1 ? "s" : "" }} en cours
          </p>
        </div>
    </header>

    <TransferList v-if="active.length" :items="active" class="flex-1 min-h-0 pr-1" />

    <div
      v-else
      class="flex-1 flex flex-col items-center justify-center text-center gap-4 min-h-0"
    >
      <Icon name="swap-horiz" :size="40" class="text-on-surface-variant/70" />
      <p class="text-body-md font-body-md text-on-surface-variant">
        Aucun transfert en cours
      </p>
      <p class="text-[11px] text-on-surface-variant/70 max-w-[240px]">
        Sélectionnez des fichiers sur l'accueil et des appareils pour envoyer.
      </p>
    </div>
  </div>
</template>