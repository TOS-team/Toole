# Guide Local (dev rapide)

Hello le BOP, ici on met les commandes minimales pour lancer le projet en local.

## 1) Lancer l'UI Python

```bash
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
.venv/bin/python src/ui/app.py
```

## 2) Compiler le coeur C (Linux)

```bash
gcc -std=c11 -Wall -Wextra -Iincludes \
  src/core/main.c src/core/app.c \
  src/platform/linux/discovery.c \
  src/platform/linux/network.c \
  src/platform/linux/file_transfert.c \
  src/platform/linux/server_runtime.c \
  -lpthread -o toole_core
```

## 3) Lancer le coeur C

```bash
./toole_core [id] [username] [ip] [tcp_port] [master|client]
```

Exemple:

```bash
./toole_core N-010 gerard 127.0.0.1 43210 client
```

## 4) Pourquoi ce mode est important

- Le coeur C tourne deja en boucle runtime.
- Le mode CLI est un socle de debug.
- Ce socle sera raccorde ensuite a l'UI Python via une couche bridge.

---

>[Architecture](architecture.md)  
>[Diagrammes](diagram.md)  
>[SRS](SRS.md)
