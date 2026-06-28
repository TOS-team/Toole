<script setup lang="ts">
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
  const btn = document.getElementById("btn-about");
  btn?.addEventListener("click", open);
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  document.getElementById("btn-about")?.removeEventListener("click", open);
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
        class="w-full max-w-[360px] bg-[rgba(18,18,22,0.96)] border border-white/12 rounded-xl p-4
               shadow-[0_14px_36px_rgba(0,0,0,0.5)]"
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-title"
      >
        <div class="flex items-center justify-between gap-3 mb-3">
          <h3 id="about-title" class="text-[15px] font-bold">À propos de Toolé</h3>
          <button
            type="button"
            aria-label="Fermer"
            class="w-[30px] h-[30px] rounded-full border border-white/12 bg-white/6
                   text-white cursor-pointer text-[18px] leading-none"
            @click="close"
          >
            &times;
          </button>
        </div>
        <p class="text-[13px] leading-relaxed">
          Toolé permet de détecter et préparer le transfert de fichiers entre
          appareils du réseau local.
        </p>
        <p class="text-[13px] leading-relaxed mt-2.5 text-dim text-[11px]">
          Version 0.2.0 &bull; Interface locale
        </p>
      </div>
    </div>
  </Teleport>
</template>
