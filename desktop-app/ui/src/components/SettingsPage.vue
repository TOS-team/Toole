<script setup lang="ts">
import { useSettingsStore, GLOW_COLORS, type ThemeMode } from "../stores/settings";

const emit = defineEmits<{ (e: "open-about"): void }>();

const settings = useSettingsStore();

const themes: { id: ThemeMode; label: string }[] = [
  { id: "auto", label: "Suivre le système" },
  { id: "dark", label: "Sombre" },
  { id: "light", label: "Clair" },
];

function colorStyle(id: string) {
  const c = GLOW_COLORS.find((x) => x.id === id);
  return c ? `rgb(${c.rgb.join(", ")})` : "#fff";
}
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col p-4 md:p-6 xl:p-8 overflow-y-auto">
    <header class="mb-6 md:mb-8 flex items-center justify-between pt-4 flex-shrink-0 mx-auto w-full max-w-[560px]">
      <div class="min-w-0">
        <h1
          class="text-headline-lg font-headline-lg text-on-background tracking-tight truncate"
        >
          Paramètres
        </h1>
        <p class="text-label-md font-label-md text-on-surface-variant mt-1">
          Personnalisez l'interface
        </p>
      </div>
    </header>

    <div class="flex flex-col gap-4 max-w-[560px] w-full mx-auto">
      <section
        class="bg-surface-container-high rounded-2xl border border-outline/50 p-5"
      >
        <h2 class="text-label-sm font-label-sm text-on-surface-variant uppercase mb-1">
          Apparence
        </h2>
        <p class="text-body-md font-body-md text-on-background mb-4">
          Thème de l'interface
        </p>

        <div class="flex flex-col gap-2">
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="t in themes"
              :key="t.id"
              type="button"
              class="w-full h-auto min-h-11 px-2 py-3 rounded-xl border text-center flex flex-col items-center justify-center gap-2 transition-colors cursor-pointer"
              :class="
                settings.theme === t.id
                  ? 'border-primary/60 bg-primary/10 text-on-surface'
                  : 'border-outline bg-surface-container-lowest text-on-surface-variant hover:border-outline-variant hover:text-on-surface'
              "
              @click="settings.theme = t.id"
            >
              <span
                class="h-4 w-4 rounded-full border-2 flex items-center justify-center shrink-0"
                :class="
                  settings.theme === t.id
                    ? 'border-primary'
                    : 'border-outline'
                "
              >
                <span
                  v-if="settings.theme === t.id"
                  class="h-2 w-2 rounded-full bg-primary"
                ></span>
              </span>
              <span class="text-label-sm font-label-sm leading-tight">{{ t.label }}</span>
            </button>
          </div>
        </div>
      </section>

      <section
        class="bg-surface-container-high rounded-2xl border border-outline/50 p-5"
      >
        <h2 class="text-label-sm font-label-sm text-on-surface-variant uppercase mb-1">
          Luminosité
        </h2>
        <div class="flex items-center justify-between gap-3 mb-3">
          <p class="text-body-md font-body-md text-on-background">
            Lueur rouge du fond
          </p>
          <span class="text-label-sm font-label-sm text-on-surface-variant shrink-0">
            {{ Math.round(settings.glow) }}%
          </span>
        </div>

        <input
          type="range"
          min="0"
          max="100"
          :value="settings.glow"
          class="w-full accent-primary cursor-pointer mb-5"
          @input="settings.glow = Number(($event.target as HTMLInputElement).value)"
        />

        <p class="text-label-md font-label-md text-on-surface-variant mb-2">
          Couleur de la lueur
        </p>
        <div class="flex flex-wrap gap-2.5">
          <button
            v-for="c in GLOW_COLORS"
            :key="c.id"
            type="button"
            :title="c.label"
            aria-label="Couleur {{ c.label }}"
            class="h-8 w-8 rounded-full border-2 transition-transform cursor-pointer"
            :class="
              settings.glowColor === c.id
                ? 'scale-110'
                : 'opacity-70 hover:scale-110 hover:opacity-100'
            "
            :style="{
              backgroundColor: colorStyle(c.id),
              borderColor: settings.glowColor === c.id ? 'var(--color-on-surface)' : 'transparent',
            }"
            @click="settings.glowColor = c.id"
          ></button>
        </div>
      </section>

      <section
        class="bg-surface-container-high rounded-2xl border border-outline/50 p-5"
      >
        <h2 class="text-label-sm font-label-sm text-on-surface-variant uppercase mb-4">
          Au sujet de
        </h2>
        <button
          type="button"
          class="w-full h-11 px-4 rounded-xl border border-outline bg-surface-container-lowest text-on-surface-variant hover:border-outline-variant hover:text-on-surface text-label-md font-label-md text-left transition-colors cursor-pointer"
          @click="emit('open-about')"
        >
          À propos de Toolé
        </button>
      </section>
    </div>
  </div>
</template>