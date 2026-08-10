import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'

export interface Transfer {
  id: string
  status: 'pending' | 'running' | 'done' | 'error' | 'cancelled'
  percent: number
  bytesSent: number
  totalBytes: number
  speed: string
  error?: string
  startTime: number
  peer?: string
  files?: string[]
}

export const useTransfersStore = defineStore('transfers', () => {
  const transfers = ref<Transfer[]>([])

  function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec > 1_048_576) return `${(bytesPerSec / 1_048_576).toFixed(1)} Mo/s`
    if (bytesPerSec > 1024) return `${(bytesPerSec / 1024).toFixed(1)} Ko/s`
    return `${bytesPerSec.toFixed(0)} o/s`
  }

  function upsert(id: string, patch: Partial<Transfer>) {
    const idx = transfers.value.findIndex(t => t.id === id)
    if (idx >= 0) {
      Object.assign(transfers.value[idx], patch)
    } else {
      transfers.value.push({
        id,
        status: 'pending',
        percent: 0,
        bytesSent: 0,
        totalBytes: 0,
        speed: '0 o/s',
        startTime: Date.now(),
        ...patch,
      })
    }
  }

  function remove(id: string) {
    transfers.value = transfers.value.filter(t => t.id !== id)
  }

  function clearHistory() {
    transfers.value = transfers.value.filter(t => t.status === 'pending' || t.status === 'running')
  }

  async function startListening() {
    await listen<string>('tool://transfer/start', (event) => {
      upsert(event.payload, { status: 'running' })
    })

    await listen<{ transfer_id: string; bytes_sent: number; total_bytes: number; percent: number }>(
      'tool://transfer/progress',
      (event) => {
        const { transfer_id, bytes_sent, total_bytes, percent } = event.payload
        const t = transfers.value.find(x => x.id === transfer_id)
        const elapsedSec = (Date.now() - (t?.startTime ?? Date.now())) / 1000
        const speed = elapsedSec > 0 ? bytes_sent / elapsedSec : 0

        upsert(transfer_id, {
          status: 'running',
          bytesSent: bytes_sent,
          totalBytes: total_bytes,
          percent,
          speed: formatSpeed(speed),
        })
      }
    )

    await listen<string>('tool://transfer/done', (event) => {
      upsert(event.payload, { status: 'done', percent: 100, speed: 'Terminé' })
    })

    await listen<string>('tool://transfer/cancel', (event) => {
      upsert(event.payload, { status: 'cancelled', speed: 'Annulé' })
    })

    await listen<{ transfer_id: string; error: string }>('tool://transfer/error', (event) => {
      upsert(event.payload.transfer_id, {
        status: 'error',
        error: event.payload.error,
        speed: 'Erreur',
      })
    })
  }

  const activeCount = computed(() => transfers.value.filter(t => t.status === 'running').length)

  return { transfers, upsert, remove, clearHistory, startListening, activeCount }
})
