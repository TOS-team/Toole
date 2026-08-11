<script setup lang="ts">
// page historique : je filtre les transferts terminés/annulés/en erreur depuis
// le store et je les affiche avec leur statut et leur date
import { computed } from "vue";
import { useTransfersStore, type Transfer } from "../stores/transfers";
import { formatSize } from "../utils";
import Icon from "./Icon.vue";

const store = useTransfersStore();

// je ne montre que les transferts finis (pas les transferts en cours)
const history = computed(() =>
  store.transfers.filter((t) =>
    ["done", "cancelled", "error"].includes(t.status),
  ),
);

function statusLabel(t: Transfer): string {
  if (t.status === "done") return "Terminé";
  if (t.status === "cancelled") return "Annulé";
  return t.error?.slice(0, 40) ?? "Erreur";
}

function statusClass(t: Transfer): string {
  if (t.status === "done") return "text-tertiary-fixed-dim bg-tertiary/15";
  if (t.status === "cancelled") return "text-on-surface-variant bg-surface-variant";
  return "text-error bg-error/15";
}

function timeLabel(ts: number): string {
  return new Date(ts).toLocaleTimeString("fr-FR", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function dateLabel(ts: number): string {
  return new Date(ts).toLocaleDateString("fr-FR", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

function transferredSize(t: Transfer): string {
  if (t.status === "done") return formatSize(t.totalBytes);
  return formatSize(t.bytesSent);
}
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col p-4 md:p-6 xl:p-8">
    <header class="mb-6 md:mb-8 flex items-start justify-between gap-4 flex-shrink-0">
      <div class="min-w-0">
        <h1
          class="text-headline-lg font-headline-lg text-on-background tracking-tight truncate"
        >
          Historique
        </h1>
        <p class="text-label-md font-label-md text-on-surface-variant mt-1">
          {{ history.length }} transfert{{ history.length > 1 ? "s" : "" }} passé{{ history.length > 1 ? "s" : "" }}
        </p>
      </div>

      <button
        v-if="history.length"
        type="button"
        title="Vider l'historique"
        class="flex items-center gap-1.5 text-label-sm font-label-sm text-on-surface-variant hover:text-error transition-colors px-2 py-1 rounded-lg cursor-pointer"
        @click="store.clearHistory()"
      >
        <Icon name="delete" :size="14" />
        Tout effacer
      </button>
    </header>

    <div v-if="history.length" class="flex-1 min-h-0 overflow-y-auto pr-1">
      <div class="flex flex-col gap-2.5">
        <div
          v-for="t in history"
          :key="t.id"
          class="bg-surface-container-high rounded-xl p-4 border border-outline/50 flex items-center gap-4"
        >
          <div
            class="h-10 w-10 rounded-lg bg-surface-container-lowest border border-outline flex items-center justify-center shrink-0"
          >
            <Icon name="folder-zip" :size="20" class="text-on-surface" />
          </div>

          <div class="flex-1 min-w-0">
            <p class="text-label-md font-label-md text-on-background truncate">
              {{ t.files?.[0] ?? `Transfert ${t.id.slice(0, 8)}` }}
              <span v-if="(t.files?.length ?? 0) > 1" class="text-on-surface-variant">
                +{{ t.files!.length - 1 }}
              </span>
            </p>
            <p class="text-[11px] text-on-surface-variant mt-0.5 truncate">
              {{ t.peer ?? "pair inconnue" }} · {{ transferredSize(t) }}
              <span v-if="t.totalBytes" class="ml-0.5">
                sur {{ formatSize(t.totalBytes) }}
              </span>
            </p>
          </div>

          <div class="flex flex-col items-end gap-1 shrink-0">
            <span
              class="text-label-sm font-label-sm px-2 py-1 rounded-md"
              :class="statusClass(t)"
            >
              {{ statusLabel(t) }}
            </span>
            <span class="text-[11px] text-on-surface-variant" :title="dateLabel(t.startTime)">
              {{ timeLabel(t.startTime) }}
            </span>
            <button
              type="button"
              title="Supprimer cette entrée"
              aria-label="Supprimer cette entrée"
              class="text-on-surface-variant hover:text-error transition-colors cursor-pointer mt-1"
              @click="store.remove(t.id)"
            >
              <Icon name="delete" :size="16" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else
      class="flex-1 flex flex-col items-center justify-center text-center gap-3 min-h-0"
    >
      <Icon name="history" :size="36" class="text-on-surface-variant opacity-50" />
      <p class="text-label-md font-label-md text-on-surface-variant">
        Aucun transfert
      </p>
      <p class="text-[11px] text-on-surface-variant/70 max-w-[240px]">
        Les transferts terminés, annulés ou en échec apparaîtront ici.
      </p>
    </div>
  </div>
</template>