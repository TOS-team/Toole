# Architecture Technique — Toolé

## Vue globale

```
┌──────────────────────────────┐
│ Frontend Tauri (React)       │
│                              │
│ - Interface utilisateur      │
│ - Boutons                    │
│ - Progression                │
│ - Liste appareils            │
└──────────────┬───────────────┘
               │ IPC
               │ invoke / events
               ▼
┌──────────────────────────────┐
│ Backend Rust                 │
│                              │
│ - UDP Discovery              │
│ - TCP Transport              │
│ - TLS Security               │
│ - File Transfer              │
│ - Hashing                    │
│ - AppState                   │
└──────────────────────────────┘
```

---

## Pourquoi Tauri ?

Tauri permet de construire une application desktop moderne avec une interface web et un backend Rust natif. Cela offre une interface moderne, de très bonnes performances, une faible consommation mémoire, et un accès système complet via Rust.

---

## Séparation Frontend / Backend

### Frontend (React + TypeScript)

Responsable uniquement de :
- l'interface graphique
- les interactions utilisateur
- l'affichage des données
- les animations
- les barres de progression

Le frontend **ne doit pas** gérer le réseau, les fichiers, la sécurité ou les sockets.

### Backend (Rust)

Responsable de :
- UDP, TCP, TLS
- fichiers, sécurité, hash
- mémoire, performances
- logique métier

---

## Communication Frontend ↔ Backend

### invoke

Permet au frontend d'appeler une fonction Rust :

```
Utilisateur clique sur Send
        ↓
invoke("start_sender")
        ↓
Rust démarre le serveur TCP
```

### events

Permet au backend d'envoyer des événements au frontend :

```
Rust reçoit un chunk fichier
        ↓
emit("transfer_progress")
        ↓
React met à jour la barre de progression
```

---

## Architecture réseau

```
                UDP Broadcast
        ┌────────────────────────┐
        │ Découverte appareils   │
        └────────────────────────┘
                    │
                    ▼
          TCP sécurisé (TLS)
                    │
      ┌─────────────┼─────────────┐
      ▼             ▼             ▼
 Receiver A    Receiver B    Receiver C
```

### UDP pour la découverte

UDP est utilisé car très léger, rapide, et parfait pour le broadcast réseau. Le sender diffuse régulièrement un packet discovery. Tous les receivers présents sur le réseau peuvent le détecter.

### TCP pour le transfert

TCP garantit l'ordre des données, la retransmission, la fiabilité, et la gestion des pertes. TCP est beaucoup plus simple à utiliser que QUIC pour une V1.

---

## Architecture async (Tokio)

Le projet utilise Tokio comme runtime asynchrone pour gérer plusieurs clients, transferts, événements réseau et UI sans créer un thread par client.

```
Tokio Runtime
│
├── UDP discovery task
├── TCP listener task
├── Client task A
├── Client task B
├── File transfer task
├── TLS task
└── UI event task
```

---

## Architecture transfert par chunks

Ne jamais charger un gros fichier entièrement en RAM :

```
File
  ↓
Buffered Reader
  ↓
Chunk Producer
  ↓
TLS Encryptor
  ↓
TCP Sender
```

Avantages : faible consommation RAM, transferts de gros fichiers, reprise future, parallélisme.

---

## Gestion mémoire

Le projet utilise streaming, buffers réutilisables, allocations minimales et architecture async. Pas de `sendfile()` dans la V1.

Optimisations futures possibles : mmap, io_uring, sendfile, vectored IO, tokio-uring.

---

## Structure des modules

| Crate | Responsabilité |
|---|---|
| `discovery` | UDP broadcast, découverte réseau, parsing packets discovery |
| `transport` | TCP, sockets, connexions, clients multiples |
| `protocol` | Structures packets, messages réseau, sérialisation |
| `security` | TLS, certificats, hash, validation identité |
| `transfer` | Lecture fichiers, chunking, progression, écriture disque |
| `core` | Logique métier, orchestration |
| `app-state` | État global : appareils connectés, transferts actifs, statistiques, sessions |
| `ui-bridge` | Communication Tauri, invoke, emit events |
