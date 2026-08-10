<script setup lang="ts">
import { computed } from "vue";
import { useTransfersStore, type Transfer } from "../stores/transfers";
import { invoke } from "../tauri";
import { formatSize } from "../utils";
import Icon from "./Icon.vue";

const props = defineProps<{ items?: Transfer[] }>();

const store = useTransfersStore();
const list = computed(() => props.items ?? store.transfers);

const hasRunning = computed(() =>
  list.value.some((t) => t.status === "running"),
);

function statusLabel(t: Transfer): string {
  if (t.status === "done") return "Terminé";
  if (t.status === "error") return t.error?.slice(0, 24) ?? "Erreur";
  if (t.status === "cancelled") return "Annulé";
  return t.speed;
}

function statusColor(t: Transfer): string {
  if (t.status === "done") return "text-tertiary-fixed-dim";
  if (t.status === "error") return "text-error";
  if (t.status === "cancelled") return "text-on-surface-variant";
  return "text-on-surface";
}

function barColor(t: Transfer): string {
  if (t.status === "done") return "bg-tertiary-fixed-dim";
  if (t.status === "error") return "bg-error";
  return "bg-primary";
}

function cardClass(t: Transfer): Record<string, boolean> {
  return {
    "border-primary/30": t.status === "running",
    "border-tertiary/60": t.status === "done",
    "border-error/60": t.status === "error",
  };
}

async function cancel(id: string) {
  await invoke("cancel_transfer", { transferId: id });
}
</script>

<template>
  <div class="flex flex-col gap-2 overflow-y-auto pr-0.5">
    <div class="flex items-center justify-between pr-1 mb-1">
      <h3 class="text-label-sm font-label-sm text-on-surface-variant uppercase">
        Transfert actif
      </h3>
      <Icon
        v-if="hasRunning"
        name="sync"
        :size="16"
        class="text-primary animate-spin"
        style="animation-duration: 3s"
      />
    </div>

    <div
      v-for="t in list"
      :key="t.id"
      class="bg-surface-container-high rounded-xl p-4 border border-outline/50"
      :class="cardClass(t)"
    >
      <div class="flex items-start gap-3">
        <div class="mt-0.5">
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
            {{ formatSize(t.bytesSent) }} / {{ formatSize(t.totalBytes) }}
          </p>
        </div>
        <button
          v-if="t.status === 'running'"
          type="button"
          aria-label="Annuler le transfert"
          title="Annuler"
          class="text-on-surface-variant hover:text-error transition-colors cursor-pointer"
          @click="cancel(t.id)"
        >
          <Icon name="close" :size="16" />
        </button>
      </div>

      <div class="w-full bg-surface-container-lowest h-1.5 rounded-full overflow-hidden mt-3 mb-2">
        <div
          class="h-full rounded-full relative transition-all duration-300"
          :class="barColor(t)"
          :style="{ width: Math.min(100, t.percent) + '%' }"
        >
          <div class="absolute right-0 top-0 bottom-0 w-4 bg-white/30 blur-sm rounded-full"></div>
        </div>
      </div>

      <div class="flex justify-between items-center text-[11px] text-on-surface-variant">
        <span>{{ t.percent }}%</span>
        <span class="truncate" :class="statusColor(t)">{{ statusLabel(t) }}</span>
      </div>
    </div>
  </div>
</template>