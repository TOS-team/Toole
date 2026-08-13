# Toolé

<p align="center">
  <img alt="Toolé" src="docs/banner.png" width="800"/>
</p>

Toolé est un logiciel de transfert de fichiers entre deux machines sur le même réseau local, sans Internet, sans clé USB, sans compte cloud. Un clic sur **Envoyer**, un clic sur **Recevoir**, et le transfert démarre.

![Licence](https://img.shields.io/github/license/TOS-team/Toole)
![Langage](https://img.shields.io/github/languages/top/TOS-team/Toole)
![CI](https://img.shields.io/github/actions/workflow/status/TOS-team/Toole/firebase.yml)
![Version](https://img.shields.io/badge/version-2.0.0-ff0033)

---

## Cas d'usage

- Partage de fichiers au Burkina
- Environnements sans accès réseau fiable
- Transferts de fichiers volumineux (>2 Go)
- Toute situation où clé USB ou réseau partagé n'est pas disponible

---

## Plateformes

- **Linux** — `.deb`, `.rpm`
- **macOS** — `.dmg` (build non notarié)
- **Windows** — `.msi`, `-setup.exe`

Les paquets sont publiés à chaque release sur la page [Releases](https://github.com/TOS-team/Toole/releases).

---

## Prérequis réseau

Toolé utilise deux ports UDP sur le réseau local :

- `58199/udp` — découverte des appareils
- `58200/udp` — transfert de fichiers (QUIC)

Les pare-feu par défaut (ex. firewalld sous Fedora) bloquent ces ports : les appareils ne se voient alors pas. Active les ports sur **chaque machine** avec :

```bash
sudo firewall-cmd --permanent --add-port=58199/udp --add-port=58200/udp
sudo firewall-cmd --reload
```

Si les appareils ne se voient toujours pas après ça, il s'agit probablement de l'**isolation des clients Wi-Fi** activée sur le routeur (souvent appelée « AP isolation » / « isolation client ») qui bloque tout trafic entre appareils en Wi-Fi. Désactive-la, ou teste avec un appareil branché en câble.

---

## Documentation

- [PRD — vision produit](docs/prd.md)
- [Architecture technique](docs/architecture.md)
- [Protocole réseau](docs/protocol.md)
- [Chiffrement et intégrité](docs/crypto.md)
- [Contribuer](CONTRIBUTING.md)

---

## Licence

[GNU GPLv3](LICENSE)
