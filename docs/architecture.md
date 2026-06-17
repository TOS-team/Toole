# Architecture Technique — Toolé

## Vue globale

```
┌──────────────────────────────────────────────────┐
│                  Toolé (workspace)                │
│                                                    │
│  ┌──────────────────────┐  ┌────────────────────┐ │
│  │       core/           │  │      app/           │ │
│  │  (bibliothèque pure)  │  │  (application Tauri)│ │
│  │                       │  │                     │ │
│  │  - error.rs           │  │  - commands.rs      │ │
│  │  - lib.rs (trait UI)  │  │    (TauriUI impl)   │ │
│  │  - utils.rs           │  │  - index.html       │ │
│  │  - discovery.rs       │  │  - main.js          │ │
│  │  - transfer.rs        │  │  - style.css        │ │
│  │  - network.rs         │  │                     │ │
│  └──────────┬────────────┘  └──────────┬─────────┘ │
│             │                           │           │
│             └───────────┬───────────────┘           │
│                         │                           │
│               Trait UI (IPC)                        │
└─────────────────────────────────────────────────────┘
```

---

## Pourquoi workspace core/ + app/ ?

`core/` est une bibliothèque Rust pure, **sans aucune dépendance Tauri**. Elle expose un **trait `UI`** générique que n'importe quel frontend peut implémenter.

`app/` est l'application Tauri qui implémente le trait `UI` en émettant des events vers le frontend HTML/JS.

Cette séparation permet :
- De tester `core/` sans Tauri (`cargo test`)
- De réutiliser `core/` pour d'autres interfaces (CLI, Android via JNI...)
- De découpler la logique métier de l'interface

---

## Trait UI — Le pont entre core et le frontend

```rust
pub trait UI: Clone + Send + 'static {
    fn log(&self, msg: &str);
    fn set_progress(&self, percent: u8, speed: &str);
    fn on_peer_found(&self, peer: Peer);
    fn on_transfer_done(&self, path: &str);
    fn on_error(&self, err: &str);
}
```

Dans `app/commands.rs`, la structure `TauriUI` implémente ce trait en émettant des events Tauri :

```rust
impl UI for TauriUI {
    fn log(&self, msg: &str) {
        self.window.emit("log", msg).unwrap();
    }
    fn set_progress(&self, percent: u8, speed: &str) {
        self.window.emit("progress", serde_json::json!({
            "percent": percent, "speed": speed
        })).unwrap();
    }
    // ...
}
```

---

## Architecture réseau

```
                    ┌──────────────┐
                    │  Sender      │
                    │              │
                    │  1. Crée     │
                    │  hotspot     │
                    │  Toole-XXXX  │
                    └──────┬───────┘
                           │ WiFi ad hoc
                           │ (hotspot ouvert)
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
       ┌──────────────┐        ┌──────────────┐
       │  Receiver    │        │  Receiver    │
       │  (V1 : un    │        │  (V2+)       │
       │   seul)      │        │              │
       └──────────────┘        └──────────────┘

       1. Sender crée hotspot + serveur TCP
       2. Receiver scanne en boucle → trouve Toole-* → se connecte
       3. Broadcast UDP sur le réseau ad hoc
       4. Connexion TCP + TLS → transfert
```

---

## Gestion du réseau (nmcli)

Le module `network.rs` utilise `nmcli` via `std::process::Command`.

### Commandes

```bash
# Créer hotspot ouvert
nmcli device wifi hotspot ifname <interface> ssid "Toole-XXXX"

# Scanner les réseaux
nmcli device wifi list --rescan yes

# Connexion
nmcli device wifi connect "Toole-XXXX"

# Déconnexion
nmcli device disconnect <interface>

# Détruire hotspot
nmcli connection delete "Toole-XXXX"
```

### Privilèges

1. Tenter `nmcli` directement
2. Si échec → `pkexec nmcli`
3. Si échec → instruction à l'écran

---

## Architecture async (Tokio)

```
Tokio Runtime
│
├── Tâche hotspot          (création / destruction réseau)
├── Tâche scan boucle      (scan WiFi côté receiver)
├── Tâche broadcast UDP    (envoi TOOLE_DISCOVER toutes les 2s)
├── Tâche écoute UDP       (réception TOOLE_HERE)
├── Tâche listener TCP     (connexion entrante)
├── Tâche transfert        (chunks + Ack + SHA-256)
└── Tâche UI events        (émission events Tauri)
```

---

## Architecture transfert par chunks

```
Fichier
   ↓
Buffered Reader (streaming)
   ↓
Chunk Producer (découpage 1 Mo)
   ↓
TLS Encryptor (rustls)
   ↓
TCP Sender
   ↓
Receiver
   ↓
TLS Decryptor
   ↓
Chunk Writer + SHA-256 progressif
   ↓
Disque
```

Avantages : faible consommation RAM, transferts de gros fichiers, reprise future (via index du dernier Ack).

---

## Structure des modules

### core/src/

| Module | Responsabilité |
|---|---|
| `lib.rs` | Trait UI, types (Peer, Mode, TransferStatus) |
| `error.rs` | ToolError (tous les types d'erreurs) |
| `utils.rs` | format_size, gen_ssid, current_hostname |
| `discovery.rs` | UDP broadcast (TOOLE_DISCOVER / TOOLE_HERE) |
| `transfer.rs` | TCP + TLS + chunks 1 Mo + Ack + SHA-256 |
| `network.rs` | nmcli hotspot, scan boucle, connexion, destruction |

### app/

| Fichier | Responsabilité |
|---|---|
| `commands.rs` | Implémentation TauriUI + commandes invoke |
| `index.html` | Interface utilisateur (2 boutons, liste, progression) |
| `main.js` | Écoute events Tauri, appelle invoke |
| `style.css` | Styles minimalistes |

---

> Voir aussi : [protocol.md](protocol.md) — le protocole réseau · [security.md](security.md) — la sécurité
