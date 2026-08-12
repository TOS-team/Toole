<script setup lang="ts">
// panneau de mise à jour : je montre l'état de la version installée et je
// propose de vérifier / installer les nouvelles versions depuis GitHub
import { ref, onMounted } from "vue";
import { useUpdaterStore } from "../stores/updater";
import Icon from "./Icon.vue";

const store = useUpdaterStore();
const hasChecked = ref(false);

// je n'affiche le "à jour" que si l'utilisateur a demandé une vérification
// explicite (le check automatique au lancement reste silencieux)
async function onCheck() {
  hasChecked.value = true;
  await store.checkForUpdate(true);
}

async function onInstall() {
  await store.install();
}

// je déclenche un check silencieux au montage du panneau, mais sans changer
// l'affichage : seules les notifications de nouvelle version comptent
onMounted(async () => {
  await store.checkForUpdate(false);
});
</script>

<template>
  <section class="bg-surface-container-high rounded-2xl border border-outline/50 p-5">
    <h2 class="text-label-sm font-label-sm text-on-surface-variant uppercase mb-1">
      Mises à jour
    </h2>
    <p class="text-body-md font-body-md text-on-background mb-4">
      {{ store.newVersion ? `Version ${store.newVersion} disponible` : "Version installée" }}
    </p>

    <div v-if="store.status === 'checking' || store.status === 'downloading' || store.status === 'installing'"
      class="flex items-center gap-2 text-label-sm font-label-sm text-on-surface-variant mb-3"
    >
      <Icon name="sync" :size="16" class="text-primary animate-spin" />
      <span>
        {{
          store.status === "checking"
            ? "Recherche de mises à jour…"
            : store.status === "installing"
              ? "Installation… redémarrage imminent"
              : `Téléchargement ${store.progressPercent()}%`
        }}
      </span>
    </div>

    <div v-if="store.status === 'available'" class="flex flex-col gap-3">
      <p v-if="store.notes" class="text-body-sm font-body-sm text-on-surface-variant max-h-24 overflow-y-auto whitespace-pre-wrap">
        {{ store.notes }}
      </p>
      <button
        type="button"
        :disabled="store.busy"
        class="w-full h-11 px-4 rounded-xl border border-primary/60 bg-primary/10 text-primary
               text-label-md font-label-md hover:bg-primary/20 disabled:opacity-50 cursor-pointer
               disabled:cursor-not-allowed transition-colors"
        @click="onInstall"
      >
        {{ store.busy ? "Installation…" : "Installer et redémarrer" }}
      </button>
    </div>

    <div v-else-if="store.status === 'up-to-date' && hasChecked"
      class="flex items-center gap-2 text-label-sm font-label-sm text-tertiary-fixed-dim mb-3"
    >
      <Icon name="check" :size="16" />
      <span>Toolé est à jour</span>
    </div>

    <button
      v-else
      type="button"
      :disabled="store.busy"
      class="w-full h-11 px-4 rounded-xl border border-outline bg-surface-container-lowest
             text-on-surface-variant hover:border-outline-variant hover:text-on-surface
             text-label-md font-label-md disabled:opacity-50 cursor-pointer disabled:cursor-not-allowed
             transition-colors"
      @click="onCheck"
    >
      Rechercher une mise à jour
    </button>

    <p v-if="store.status === 'error'" class="mt-3 text-label-sm font-label-sm text-error break-words">
      Erreur de mise à jour : {{ store.error }}
    </p>
  </section>
</template>