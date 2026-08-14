<script setup lang="ts">
// popup de demande de transfert entrant : bien visible au-dessus de toute
// l'interface, avec un grand choix accepter / refuser. Elle suit le premier
// transfert 'incoming' en attente ; dès qu'il est résolu, la suivante
// s'affiche à sa place.
import { computed } from "vue";
import { useTransfersStore } from "../stores/transfers";
import { formatSize } from "../utils";
import Icon from "./Icon.vue";

const store = useTransfersStore();

// je prends la plus ancienne demande encore en attente de validation
const incoming = computed(
  () => store.transfers.find((t) => t.status === "incoming") ?? null,
);

// je réponds à la demande avec un état optimiste : la carte passe en
// « running » (accepté) ou « refused », la popup disparaît ; en cas
// d'erreur du pont je restaure le statut 'incoming'
async function respond(accepted: boolean) {
  if (!incoming.value) return;
  const id = incoming.value.id;
  store.upsert(id, {
    status: accepted ? "running" : "refused",
    speed: accepted ? "Envoi…" : "Refusé",
  });
  try {
    await store.respond(id, accepted);
  } catch (e) {
    console.error("respond error:", e);
    store.upsert(id, { status: "incoming" });
  }
}

const fileCount = computed(() => incoming.value?.files?.length ?? 1);
</script>

<template>
  <Teleport to="body">
    <div
      v-if="incoming"
      class="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center p-[18px] z-30"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="incoming-title"
    >
      <div
        class="w-full max-w-[420px] bg-surface-container-high border border-outline-variant rounded-2xl p-6 active-shadow"
      >
        <div class="flex items-center gap-3 mb-4">
          <span
            class="w-12 h-12 rounded-xl bg-primary/15 text-primary flex items-center justify-center shrink-0"
          >
            <Icon name="folder-zip" :size="24" />
          </span>
          <div class="min-w-0">
            <h3
              id="incoming-title"
              class="text-headline-md font-headline-md text-on-background truncate"
            >
              Transfert entrant
            </h3>
            <p class="text-label-md font-label-md text-on-surface-variant truncate">
              de {{ incoming.peer || "inconnu" }} · {{ formatSize(incoming.totalBytes) }}
            </p>
          </div>
        </div>

        <div
          class="bg-surface-container-lowest border border-outline/50 rounded-xl p-3 mb-5 max-h-44 overflow-y-auto"
        >
          <p class="text-label-sm font-label-sm text-on-surface-variant mb-1.5 uppercase">
            {{ fileCount }} fichier{{ fileCount > 1 ? "s" : "" }}
          </p>
          <ul v-if="incoming.files?.length" class="flex flex-col gap-1">
            <li
              v-for="f in incoming.files.slice(0, 8)"
              :key="f"
              class="text-[12px] text-on-surface truncate flex items-center gap-2"
            >
              <Icon name="file" :size="14" class="text-on-surface-variant shrink-0" />
              <span class="truncate">{{ f }}</span>
            </li>
          </ul>
          <p v-else class="text-[12px] text-on-surface-variant">(aucun nom fourni)</p>
        </div>

        <div class="flex flex-col gap-2">
          <button
            type="button"
            class="w-full h-12 bg-primary text-on-primary rounded-xl flex items-center justify-center gap-2 text-label-md font-label-md hover:opacity-90 transition-opacity cursor-pointer"
            @click="respond(true)"
          >
            <Icon name="check" :size="18" />
            Accepter le transfert
          </button>
          <button
            type="button"
            class="w-full h-12 bg-surface-container-lowest border border-outline rounded-xl flex items-center justify-center gap-2 text-label-md font-label-md text-on-surface-variant hover:text-error hover:border-error/50 transition-colors cursor-pointer"
            @click="respond(false)"
          >
            <Icon name="close" :size="18" />
            Refuser
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>