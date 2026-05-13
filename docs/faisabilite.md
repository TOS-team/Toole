<<<<<<< HEAD
# Faisabilité Technique
 - Language C (pointeur,thread,struct,etc...)
 - Resources humaines deja disponible

# Faisabilité Temporelle
 - Multiplateforme (Linux,Windows): 2 jours
 - Thread du proximity pairing: 7 jours
 - Thread de tranferts de fichiers: 7 jours
 - Thread du TUI: 3 jours
---
 Total = 19 * 1.5 = 28 jours

# Faisabilité Opérationnele
 >les taches seront par le nombre de contributeurs et chacun sera tenu de soumettre son travail au moment venu et dans les delais qui lui seront accordé.
 
 ---
 >[Product Requirement Document (PRD)](PRD.md)
=======
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
>>>>>>> 4f2ec6ce32eb8f8092ad46be7fd76220b4f353dd
