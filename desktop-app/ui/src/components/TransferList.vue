<template>
  <div v-if="store.transfers.length" class="transfer-panel">
    <h3 class="transfer-title">Transferts</h3>
    <div
      v-for="t in store.transfers"
      :key="t.id"
      class="transfer-item"
      :class="{ done: t.status === 'done', error: t.status === 'error', cancelled: t.status === 'cancelled' }"
    >
      <div class="transfer-header">
        <span class="transfer-id">{{ t.id.slice(0, 8) }}…</span>
        <span class="transfer-status">{{ statusLabel(t) }}</span>
      </div>

      <div class="progress-track">
        <div class="progress-fill" :style="{ width: t.percent + '%' }" />
      </div>

      <div class="transfer-meta">
        <span>{{ formatSize(t.bytesSent) }} / {{ formatSize(t.totalBytes) }}</span>
        <span>{{ t.speed }}</span>
      </div>

      <button
        v-if="t.status === 'running'"
        class="btn-cancel"
        @click="cancel(t.id)"
      >
        Annuler
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useTransfersStore } from '../stores/transfers'
import { invoke } from '@tauri-apps/api/core'

const store = useTransfersStore()

function statusLabel(t: typeof store.transfers[0]) {
  if (t.status === 'done') return '✅ Terminé'
  if (t.status === 'error') return '❌ ' + (t.error?.slice(0, 30) ?? 'Erreur')
  if (t.status === 'cancelled') return '🚫 Annulé'
  return `⏳ ${t.percent}%`
}

function formatSize(bytes: number): string {
  if (bytes > 1_073_741_824) return (bytes / 1_073_741_824).toFixed(2) + ' Go'
  if (bytes > 1_048_576) return (bytes / 1_048_576).toFixed(1) + ' Mo'
  if (bytes > 1024) return (bytes / 1024).toFixed(1) + ' Ko'
  return bytes + ' o'
}

async function cancel(id: string) {
  await invoke('cancel_transfer', { transferId: id })
}
</script>

<style scoped>
.transfer-panel {
  margin-top: 1rem;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  backdrop-filter: blur(10px);
}
.transfer-title {
  margin: 0 0 0.75rem;
  font-size: 0.875rem;
  font-weight: 600;
  color: #e2e8f0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.transfer-item {
  margin-bottom: 0.75rem;
  padding: 0.75rem;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}
.transfer-item.done { border-left: 3px solid #22c55e; }
.transfer-item.error { border-left: 3px solid #ef4444; }
.transfer-item.cancelled { border-left: 3px solid #f59e0b; }

.transfer-header {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: #94a3b8;
  margin-bottom: 0.5rem;
}
.progress-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  border-radius: 3px;
  transition: width 0.3s ease;
}
.transfer-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 0.5rem;
  font-size: 0.75rem;
  color: #94a3b8;
}
.btn-cancel {
  margin-top: 0.5rem;
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  color: #fecaca;
  background: rgba(239, 68, 68, 0.2);
  border: 1px solid rgba(239, 68, 68, 0.4);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}
.btn-cancel:hover {
  background: rgba(239, 68, 68, 0.4);
}
</style>
