# Diagram expliquant les mecanismes complexes du projet

Hello le BOP, ici je mets les schemas textuels pour qu'on ait la meme lecture du systeme.

## 1) Discovery UDP

```text
[Node A] --beacon UDP--> [LAN]
  payload: toole|id|username|ip|tcp_port|role|cluster_id|master_ip|master_port|message

[Node B] --hear()--> parse + dedoublonnage + maj liste devices[]
[Node B] --cleaner()--> supprime les noeuds muets > 10s
```

## 2) Connexion normale au Master

```text
[Client nouveau]
   | recoit beacon role=MASTER
   v
connect_to(master_ip, master_port)
   |
   +--> network_send_hello()
   +--> boucle heartbeat (network_send_heartbeat)
```

## 3) Connexion via voisin (mode relaye)

```text
[Client nouveau] --TCP--> [Voisin client joignable]
      |                          |
      |-- RELAY_REQUEST -------->|
      |<-- RELAY_RESPONSE -------|  (master_ip, master_port, cluster_id)
      |
      +--> connect_to(master_ip, master_port)
```

## 4) Perte du Master et election

```text
Boucle client:
  runtime_client_step(...)
      |
      +--> timeout / recv==0 / erreur
              |
              v
           state = ELECTION
              |
              +--> runtime_elect_master_from_devices(...)
                      regle: plus petit id gagne
```

## 5) Issue de l'election

```text
Si self.id == winner.id:
   -> state = MASTER
   -> annonce potentielle aux clients (MASTER_ANNOUNCE)

Sinon:
   -> tentative de reco directe au winner
   -> si echec: runtime_try_reconnect_from_devices (fallback voisin relay)
   -> state = CLIENT
```

## 6) Transfert de fichier TCP

```text
send_file():
  send_struct(name_len + file_size + filename)
  send(chunk1)
  send(chunk2)
  ...

recv_file():
  recv_struct(...)
  recv(chunks...) -> write(destination/filename)
```

## 7) Matrice des modules (qui fait quoi)

| Module | Role principal |
|---|---|
| `discovery.c` | Presence UDP + ecoute + nettoyage liste |
| `network.c` | Controle TCP (HELLO/HB/RELAY/ELECTION) |
| `file_transfert.c` | Flux fichier TCP |
| `server_runtime.c` | Orchestration failover et transitions |

---

> Vue narrative complete: [architecture.md](architecture.md)
