<script setup lang="ts">
// modale "à propos" : ouverte par le bouton des paramètres, fermée par
// Échap ou en cliquant sur le fond
import { ref, onMounted, onUnmounted } from "vue";

const isOpen = ref(false);

function open() {
  isOpen.value = true;
}

function close() {
  isOpen.value = false;
}

function onBackdropClick(e: MouseEvent) {
  if ((e.target as HTMLElement).id === "about-modal") close();
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") close();
}

defineExpose({ open, close });

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div
      id="about-modal"
      class="fixed inset-0 bg-black/55 backdrop-blur-md flex items-center justify-center p-[18px] z-20"
      :class="isOpen ? '' : 'hidden'"
      @click="onBackdropClick"
    >
      <div
        class="w-full max-w-[360px] bg-surface-container-high border border-outline-variant rounded-2xl p-4 active-shadow"
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-title"
      >
        <div class="flex items-center justify-between gap-3 mb-3">
          <h3 id="about-title" class="text-headline-md font-headline-md text-on-background">
            À propos de Toolé
          </h3>
          <button
            type="button"
            aria-label="Fermer"
            class="w-[30px] h-[30px] rounded-full border border-outline-variant bg-surface-variant
                   text-on-surface-variant cursor-pointer text-[18px] leading-none
                   hover:text-on-surface hover:bg-surface-container-highest transition-colors"
            @click="close"
          >
            &times;
          </button>
        </div>
        <p class="text-body-md font-body-md text-on-surface leading-relaxed">
          Toolé est un logiciel de transfert de fichiers entre deux
          machines sur le même réseau local, sans Internet, sans clé USB,
          sans compte cloud.
        </p>
        <p class="text-label-md font-label-md text-on-surface-variant mt-2.5">
          Version 2.0.0 &bull; Interface locale &bull; Licence GPL-3.0
        </p>
      </div>
    </div>
  </Teleport>
</template>