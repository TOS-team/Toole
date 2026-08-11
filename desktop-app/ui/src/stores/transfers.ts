import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
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

const KEY = 'toole.transfers'
const MAX_HISTORY = 200

const TERMINAL: Transfer['status'][] = ['done', 'error', 'cancelled']

function loadHistory(): Transfer[] {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    const loaded: Transfer[] = []
    for (const t of parsed) {
      if (!t || typeof t.id !== 'string') continue
      const status = TERMINAL.includes(t.status) ? t.status : 'error'
      loaded.push({
        id: t.id,
        status,
        percent: typeof t.percent === 'number' ? t.percent : 0,
        bytesSent: typeof t.bytesSent === 'number' ? t.bytesSent : 0,
        totalBytes: typeof t.totalBytes === 'number' ? t.totalBytes : 0,
        speed: status === 'error' ? 'Erreur' : typeof t.speed === 'string' ? t.speed : 'Terminé',
        error:
          !TERMINAL.includes(t.status)
            ? 'Interrompu au redémarrage'
            : typeof t.error === 'string'
              ? t.error
              : undefined,
        startTime: typeof t.startTime === 'number' ? t.startTime : Date.now(),
        peer: typeof t.peer === 'string' ? t.peer : undefined,
        files: Array.isArray(t.files)
          ? t.files.filter((f: unknown) => typeof f === 'string')
          : undefined,
      })
    }
    return loaded.slice(-MAX_HISTORY)
  } catch {
    return []
  }
}

export const useTransfersStore = defineStore('transfers', () => {
  const transfers = ref<Transfer[]>(loadHistory())

  let saveTimer: ReturnType<typeof setTimeout> | null = null
  function save() {
    try {
      const settled = transfers.value.filter(t => TERMINAL.includes(t.status))
      localStorage.setItem(KEY, JSON.stringify(settled.slice(-MAX_HISTORY)))
    } catch {
      /* stockage indisponible : on ignore */
    }
  }

  watch(
    transfers,
    () => {
      if (saveTimer) clearTimeout(saveTimer)
      saveTimer = setTimeout(save, 300)
    },
    { deep: true },
  )

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

    await listen<{ transfer_id: string; peer: string; bytes: number; files: string[] }>(
      'tool://transfer/received',
      (event) => {
        upsert(event.payload.transfer_id, {
          status: 'done',
          percent: 100,
          bytesSent: event.payload.bytes,
          totalBytes: event.payload.bytes,
          speed: 'Terminé',
          peer: event.payload.peer,
          files: event.payload.files,
        })
      },
    )
  }

  const activeCount = computed(() => transfers.value.filter(t => t.status === 'running').length)

  return { transfers, upsert, remove, clearHistory, startListening, activeCount }
})
