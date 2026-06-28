<script setup lang="ts">
defineProps<{ hostname: string }>();
</script>

<template>
  <div class="flex items-center gap-3 pb-1">
    <img
      src="/assets/img/sticker.png"
      alt="Toolé logo"
      class="w-14 h-14 object-contain"
    />
    <h2 class="flex-1 text-[16px] font-bold tracking-tight">
      Bienvenue
      <span class="block text-[11px] text-red-brand font-bold mt-0.5">
        {{ hostname || "inconnu" }}
      </span>
    </h2>
    <button
      id="btn-about"
      type="button"
      aria-label="À propos"
      title="À propos"
      class="w-8 h-8 rounded-full text-sm cursor-pointer
             bg-white/10 border border-white/20 shrink-0
             backdrop-blur-md saturate-[160%]
             shadow-[0_2px_8px_rgba(0,0,0,0.15),inset_0_1px_0_rgba(255,255,255,0.25)]
             transition-all duration-150 hover:brightness-125 hover:scale-105 active:scale-95
             flex items-center justify-center"
    >
      <img src="/assets/icons/icone-dinformation.png" alt="À propos" class="w-[18px] h-[18px]" style="filter: brightness(0) invert(1)" />
    </button>
  </div>
</template>
