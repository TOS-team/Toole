<<<<<<< HEAD
# PRD (Product Requirement Document)

## Context 
>Le transfere de  fichier entre ordinateur hors ligne est lente ou souvent complexe à mettre en place,**Toolé** veut faciliter cette manoeuvre.[Voir plus](brouillon.md)

## Les personnes concernées
- Noaga un etudiant qui veut envoyer des documents à son camarade dans un amphi avec une connexion tres faible et de disposant pas de clé USB.
- Tinbnooma une expert en cybersécurité veut transférer un fichier sensible entre avec son collegue sans laisser aucune trace sur un suport physique ou sur internet.

## L'inventaire des fonctionnalités
- F-001: l'appairage à proximité, raproher les deux ordinateurs suffit à etablir une connexion
- F-002: Transfere de fichiers securisé
- F-003: Une interface TUI (Text-based User Interface)    

## Specifications detaillés
### F-001
- L'utilisateur peut accepter ou refuser un apairage meme si on est dans un cas d'apairage de proximité
- L'utilisateur aura toujours le choix de d'activer ou non l'apairage de proximité

## Exigences non-fonctionnelles
- Les transferes doivent etre rapide

---
>[Software Requirement Specifications (SRS)](SRS.md)

=======
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
>>>>>>> 4f2ec6ce32eb8f8092ad46be7fd76220b4f353dd
