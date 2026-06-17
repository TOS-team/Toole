# Protocole Réseau — Toolé

## UDP — Découverte (port 5199)

L'UDP est utilisé pour l'échange de présence une fois que le receiver est connecté au hotspot.

| Direction | Paquet | Description |
|---|---|---|
| Sender → broadcast | `TOOLE_DISCOVER` | Diffusé toutes les 2s sur `255.255.255.255:5199` |
| Receiver → Sender | `TOOLE_HERE:<hostname>` | Réponse unicast, ex: `TOOLE_HERE:PC-Gerard` |

---

## TCP + TLS — Transfert (port 5200)

Tous les paquets suivants transitent par une connexion TCP chiffrée en TLS 1.3.

### Metadata

Envoyé par le sender au receiver. JSON encodé en UTF-8, terminé par `\n`.

```json
{
  "filename": "rapport.pdf",
  "size": 104857600,
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "chunk_size": 1048576
}
```

### Ack metadata

Le receiver envoie un seul octet : `0x01` (ACK) pour confirmer réception des métadonnées.

### Chunk

Binaire. Format :

| Champ | Taille | Détail |
|---|---|---|
| chunk_index | 4 octets | u32 big-endian |
| data | N octets | Contenu du chunk (N = chunk_size sauf dernier) |

### Ack chunk

Le receiver répond avec l'index du chunk reçu (4 octets, u32 big-endian).

### Complete

Envoyé après le dernier chunk. JSON encodé en UTF-8, terminé par `\n`.

```json
{
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

### FinalAck

Le receiver répond `0x01` (OK, SHA-256 vérifié) ou `0x00` (REJET, SHA-256 mismatch).

---

## Séquence complète

```
Sender                              Receiver
  │                                     │
  │─── TOOLE_DISCOVER (UDP) ──────────►│
  │◄── TOOLE_HERE:PC-Gerard (UDP) ─────│
  │                                     │
  │=== Connexion TCP + TLS handshake ==>│
  │                                     │
  │─── Metadata (JSON) ────────────────►│
  │◄── Ack (0x01) ─────────────────────│
  │─── Chunk(0) ──────────────────────►│
  │◄── Ack(0) ─────────────────────────│
  │─── Chunk(1) ──────────────────────►│
  │◄── Ack(1) ─────────────────────────│
  │─── ... ───────────────────────────►│
  │─── Complete (JSON) ───────────────►│
  │◄── FinalAck (0x01) ───────────────│
```

---

> [Architecture technique](architecture.md) | Lire ensuite : [Chiffrement et intégrité](crypto.md)
