# Toolé

![Toolé](logo.png)

**Transfert de fichiers P2P en réseau local, sans internet.**

Toolé est un système de transfert de fichiers pair-à-pair sur réseau local. Pas de cloud, pas de serveur central — découverte automatique des appareils, élection d'un master, et transfert direct entre nœuds.

## Fonctionnalités

- **Découverte automatique** des appareils sur le LAN via beacon UDP
- **Cluster auto-organisé** avec élection de master et failover automatique
- **Transfert de fichiers** avec vérification CRC32
- **Interface graphique** (CustomTkinter) avec bridge C pour les opérations réseau

## Releases

Les archives pré-compilées sont disponibles dans la section [Releases](https://github.com/anomalyco/Toole/releases).

## Contribuer

Consultez [CONTRIBUTING.md](CONTRIBUTING.md) pour les axes majeurs de contribution (réécriture Rust, chiffrement, etc.) et la procédure de build.

## Licence

[MIT](LICENSE)
