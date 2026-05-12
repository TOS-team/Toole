# PRD (Product Requirement Document)

Hello le BOP, ici on pose le besoin produit de Toolé en version claire.

## 1) Contexte

Le transfert de fichiers hors ligne entre ordinateurs reste souvent lent ou compliqué (clé USB, manipulations manuelles, dépendance internet).  
**Toolé** veut rendre ça simple avec un réseau local auto-organisé.

> Vue problème détaillée: [brouillon.md](brouillon.md)

## 2) Personnes concernées

- **Noaga**: étudiant qui veut partager rapidement des documents dans un environnement avec réseau faible.
- **Tinbnooma**: profil cybersécurité qui veut transférer des fichiers sensibles sans support physique.

## 3) Inventaire des fonctionnalités

- **F-001**: Appairage à proximité (découverte automatique des appareils).
- **F-002**: Transfert de fichier sécurisé et fiable.
- **F-003**: Interface de contrôle simple (TUI cible, UI Python provisoire pendant le dev).

## 4) Spécifications détaillées

### F-001 — Appairage à proximité

- Le système détecte les appareils en LAN via discovery UDP.
- L'utilisateur peut accepter ou refuser une demande de connexion.
- L'appairage automatique peut être activé/désactivé.

### F-002 — Transfert et continuité de service

- Le système établit une connexion TCP stable entre nœuds.
- Le cluster s'organise autour d'un Master logique.
- Si le Master tombe, une élection automatique relance le cluster.
- Un nœud peut rejoindre le Master directement ou via un voisin relay.

### F-003 — Interface opérable

- L'interface doit rester simple pour envoyer/recevoir un fichier.
- L'utilisateur doit avoir des retours clairs (connexion, progression, statut).

## 5) Exigences non fonctionnelles

- Découverte réseau rapide.
- Transfert fiable sans corruption de fichier.
- Comportement prévisible en cas de panne (failover).
- Lisibilité du code et des logs techniques.

---

>[Software Requirement Specifications (SRS)](SRS.md)  
>[Architecture technique](architecture.md)
