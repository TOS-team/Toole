// je gère l'historique et l'état des transferts. Chaque événement émis par le
// processus Rust (progression, erreur, annulation...) met à jour un transfert,
// et je persiste les transferts terminés dans localStorage pour les afficher
// après un redémarrage.
import { defineStore } from "pinia"
import { ref, computed, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '../tauri'

export interface Transfer {
  id: string
  status: 'pending' | 'incoming' | 'running' | 'done' | 'error' | 'cancelled' | 'refused'
  percent: number
  bytesSent: number
  totalBytes: number
  speed: string
  error?: string
  startTime: number
  peer?: string
  files?: string[]
  fileProgress?: FileProgress[]
}

export interface FileProgress {
  name: string
  bytesSent: number
  totalBytes: number
  percent: number
}

const KEY = 'toole.transfers'
const MAX_HISTORY = 200

const TERMINAL: Transfer['status'][] = ['done', 'error', 'cancelled', 'refused']

// je relis l'historique persisté et je le valide entrée par entrée : les
// statuts inconnus deviennent des erreurs et les champs manquants ont des
// valeurs par défaut. Je tronque à MAX_HISTORY pour éviter de stocker trop.
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
        fileProgress: Array.isArray(t.fileProgress)
          ? t.fileProgress
              .filter(
                (f: unknown) =>
                  f &&
                  typeof f === 'object' &&
                  typeof (f as FileProgress).name === 'string',
              )
              .map((f: FileProgress) => ({
                name: f.name,
                bytesSent: typeof f.bytesSent === 'number' ? f.bytesSent : 0,
                totalBytes: typeof f.totalBytes === 'number' ? f.totalBytes : 0,
                percent: typeof f.percent === 'number' ? f.percent : 0,
              }))
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
  // je ne persiste que les transferts terminés (l'historique), pas les
  // transferts en cours, pour ne pas ressusciter un état obsolète
  function save() {
    try {
      const settled = transfers.value.filter(t => TERMINAL.includes(t.status))
      localStorage.setItem(KEY, JSON.stringify(settled.slice(-MAX_HISTORY)))
    } catch {
      /* stockage indisponible : on ignore */
    }
  }

  // je débounce la sauvegarde de 300ms pour ne pas écrire à chaque octet
  watch(
    transfers,
    () => {
      if (saveTimer) clearTimeout(saveTimer)
      saveTimer = setTimeout(save, 300)
    },
    { deep: true },
  )

  // je formate un débit octets/seconde en unités décimales (×1000), comme
  // formatSize, pour une métrique cohérente sur toute l'interface
  function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec > 1000 * 1000) return `${(bytesPerSec / (1000 * 1000)).toFixed(1)} Mo/s`
    if (bytesPerSec > 1000) return `${(bytesPerSec / 1000).toFixed(1)} Ko/s`
    return `${bytesPerSec.toFixed(0)} o/s`
  }

  // je crée le transfert s'il n'existe pas (en pending) puis j'applique le
  // patch des champs reçus de l'événement
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

  // j'efface l'historique mais je garde les transferts toujours en cours
  function clearHistory() {
    transfers.value = transfers.value.filter(t => t.status === 'pending' || t.status === 'running')
  }

  // je m'abonne aux événements de transfert émis par le processus Rust
  async function startListening() {
    await listen<string>('tool://transfer/start', (event) => {
      upsert(event.payload, { status: 'running' })
    })

    // un transfert entrant attend la validation de l'utilisateur : je crée la
    // carte en statut 'incoming' avec les infos du lot (émetteur, taille,
    // fichiers) pour afficher les boutons accepter / refuser
    await listen<{
      transfer_id: string
      sender: string
      total_bytes: number
      files: string[]
    }>('tool://transfer/incoming', (event) => {
      const { transfer_id, sender, total_bytes, files } = event.payload
      upsert(transfer_id, {
        status: 'incoming',
        peer: sender,
        totalBytes: total_bytes,
        files,
        percent: 0,
        bytesSent: 0,
        speed: 'En attente de validation…',
      })
    })

    await listen<string>('tool://transfer/refused', (event) => {
      upsert(event.payload, { status: 'refused', speed: 'Refusé' })
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

    // je mets à jour la barre de progression du fichier en cours, en la
    // créant si on ne l'a pas encore vue
    await listen<{
      transfer_id: string
      file_name: string
      file_bytes_sent: number
      file_total_bytes: number
      percent: number
    }>('tool://transfer/file_progress', (event) => {
      const { transfer_id, file_name, file_bytes_sent, file_total_bytes, percent } = event.payload
      const t = transfers.value.find(x => x.id === transfer_id)
      if (!t) return
      const list = t.fileProgress ?? []
      const idx = list.findIndex(f => f.name === file_name)
      const entry = { name: file_name, bytesSent: file_bytes_sent, totalBytes: file_total_bytes, percent }
      if (idx >= 0) list[idx] = entry
      else list.push(entry)
      upsert(transfer_id, { status: 'running', fileProgress: [...list] })
    })

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

  // je réponds à une demande d'acceptation (accepter / refuser le transfert)
  async function respond(id: string, accepted: boolean) {
    await invoke("respond_transfer", { transferId: id, accepted })
  }

  // je compte les transferts encore actifs (badge de la barre latérale) :
  // les envois en attente de validation et les demandes entrantes comptent
  const activeCount = computed(
    () =>
      transfers.value.filter(t =>
        t.status === 'running' || t.status === 'pending' || t.status === 'incoming',
      ).length,
  )

  return {
    transfers,
    upsert,
    remove,
    clearHistory,
    startListening,
    respond,
    activeCount,
  }
})
