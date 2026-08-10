<script setup lang="ts">
import Icon from "./Icon.vue";

defineProps<{ hostname: string; active: string }>();
const emit = defineEmits<{ (e: "navigate", id: string): void }>();

const items = [
  { id: "home", label: "Accueil", icon: "send" },
  { id: "history", label: "Historique", icon: "history" },
  { id: "transfers", label: "Transferts", icon: "swap-horiz" },
] as const;

const bottomItems = [
  { id: "settings", label: "Paramètres", icon: "settings" },
] as const;
</script>

<template>
  <nav
    class="flex flex-col w-[80px] bg-surface-container border border-outline-variant active-shadow rounded-2xl py-6 flex-shrink-0 z-20 items-center justify-between"
  >
    <div class="w-full flex flex-col items-center">
      <div class="mb-8">
        <img
          src="/assets/img/sticker.png"
          alt="Logo Toolé"
          class="w-[52px] h-[30px] object-contain"
        />
      </div>

      <ul class="flex flex-col w-full space-y-4 items-center">
        <li v-for="item in items" :key="item.id" class="w-full px-2">
          <button
            type="button"
            :aria-current="active === item.id ? 'page' : undefined"
            :title="item.label"
            class="relative flex flex-col items-center justify-center w-full h-14 rounded-xl transition-all group cursor-pointer"
            :class="
              active === item.id
                ? 'text-primary bg-primary/10'
                : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-variant/50'
            "
            @click="emit('navigate', item.id)"
          >
            <span
              v-if="active === item.id"
              class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-primary rounded-r-full neon-glow"
            ></span>
            <Icon
              :name="item.icon"
              :size="24"
              class="group-hover:scale-110 transition-transform"
            />
            <span
              class="text-[10px] mt-1 opacity-0 group-hover:opacity-100 absolute -bottom-5 transition-opacity whitespace-nowrap bg-surface-container px-2 py-1 rounded text-on-surface z-10"
            >
              {{ item.label }}
            </span>
          </button>
        </li>
      </ul>
    </div>

    <div class="w-full px-2 flex flex-col items-center">
      <button
        v-for="item in bottomItems"
        :key="item.id"
        type="button"
        :aria-current="active === item.id ? 'page' : undefined"
        :title="item.label"
        class="relative flex flex-col items-center justify-center w-full h-14 rounded-xl transition-all group cursor-pointer"
        :class="
          active === item.id
            ? 'text-primary bg-primary/10'
            : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-variant/50'
        "
        @click="emit('navigate', item.id)"
      >
        <span
          v-if="active === item.id"
          class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-primary rounded-r-full neon-glow"
        ></span>
        <Icon
          :name="item.icon"
          :size="24"
          class="group-hover:scale-110 transition-transform"
        />
        <span
          class="text-[10px] mt-1 opacity-0 group-hover:opacity-100 absolute -bottom-5 transition-opacity whitespace-nowrap bg-surface-container px-2 py-1 rounded text-on-surface z-10"
        >
          {{ item.label }}
        </span>
      </button>
    </div>
  </nav>
</template>