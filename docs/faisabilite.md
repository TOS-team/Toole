# Etude de faisabilite

Hello le BOP, ici on valide si Toolé est faisable techniquement, humainement et operationnellement.

## 1) Faisabilite technique

- Le socle C est adapte (sockets, threads, structures, I/O fichiers).
- L'architecture UDP discovery + TCP controle/transfert est implementable.
- Le failover Master/Client est viable avec une regle d'election deterministe.
- Les modules sont separes (`discovery`, `network`, `file_transfert`, `server_runtime`) donc maintenables.

## 2) Faisabilite operationnelle

- Le travail est decoupable par module entre contributeurs.
- Le projet peut avancer par increments testables (smoke tests reseau/fichier/failover).
- La documentation est maintenant alignee avec l'etat du code Linux.

## 3) Faisabilite de livraison (etat actuel)

- **Pret**: socle Linux reseau, contrat headers, tests smoke locaux.
- **A finir**: parity Windows, industrialisation build/CI, liaison native propre avec l'UI Python.

## 4) Risques restants

- Ecarts entre comportement runtime reel et tests unitaires.
- Regression lors de l'integration finale UI + coeur reseau.
- Support multi-OS incomplet tant que Windows n'est pas aligne.

---

>[PRD](PRD.md)  
>[SRS](SRS.md)  
>[Architecture](architecture.md)
