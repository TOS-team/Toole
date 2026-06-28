# Protocole Réseau — Toolé

## UDP — Découverte (port 58199)

L'UDP est utilisé pour la découverte des appareils sur le réseau local.

| Direction | Paquet | Description |
|---|---|---|
| Broadcast | `TOOLE_DISCOVERY` | Diffusé toutes les 3s sur `255.255.255.255:58199` |
| Unicast | `TOOLE_HERE:<hostname>` | Réponse à l'expéditeur, ex: `TOOLE_HERE:PC-Gerard` |

### Détails

- **Port** : 58199
- **Intervalle de broadcast** : 3 secondes
- **Timeout d'expiration d'un pair** : 9 secondes sans réponse
- **Format du hostname** : nom de la machine (via `gethostname`)
- **Filtrage** : un appareil ne s'ajoute pas lui-même (même hostname + même IP)

---

## TCP + TLS — Transfert (port 5200) — NON IMPLÉMENTÉ

Le protocole de transfert de fichiers est en phase de conception. Voir `core/src/transfer.rs` pour le plan détaillé.

---

## Séquence actuelle

```
Appareil A                           Appareil B
  │                                     │
  │─── TOOLE_DISCOVERY (UDP) ──────────►│
  │◄── TOOLE_HERE:PC-B (UDP) ──────────│
  │                                     │
  │─── TOOLE_DISCOVERY (UDP) ──────────►│
  │◄── TOOLE_HERE:PC-B (UDP) ──────────│
  │                                     │
  │  (toutes les 3s)                    │
  │  (si pas de réponse pendant 9s,     │
  │   le pair est retiré de la liste)   │
```

---

> [Architecture technique](architecture.md) | Lire ensuite : [Chiffrement et intégrité](crypto.md)
