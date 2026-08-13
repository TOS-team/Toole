# Utilisation de Toolé

## Interface

L'application se compose de trois zones :

- **Barre latérale gauche** : navigation entre Accueil, Transferts, Historique
  et Paramètres.
- **Zone principale** : contenu de la page active.
- **Panneau droit** : liste des appareils détectés + bouton **Transférer**.

### Titlebar personnalisée

La fenêtre n'a pas de bordure système. Utilisez la **barre de titre** en haut
pour déplacer la fenêtre (glisser), et les boutons **Réduire** / **Fermer** à
droite.

## Découverte des appareils

- Au démarrage, Toolé écoute sur le port **58199** et diffuse un message
  `TOOLE_DISCOVERY` en broadcast **toutes les 3 secondes**.
- Tout appareil Toolé qui reçoit ce message répond `TOOLE_HERE:<device_id>`.
- Un appareil qui ne répond plus pendant **9 secondes** disparaît de la liste.

L'identifiant d'un appareil (`device_id`) est **stable** : il combine le nom de
la machine et un suffixe court (base32 Crockford) généré une fois puis stocké.

### Rafraîchir

Cliquez sur l'icône **rafraîchir** (à côté de « Appareils (n) ») pour relancer
la découverte immédiatement et purger la liste.

### Sélection multiple

- Cliquez sur un appareil pour le **sélectionner** (il passe en surbrillance).
- Cliquez à nouveau pour le **désélectionner**.
- Vous pouvez sélectionner **plusieurs appareils** : l'envoi se fera vers tous
  en parallèle.

## Envoyer des fichiers

### 1. Ajouter des fichiers

Plusieurs méthodes :

| Méthode | Comment faire |
|---|---|
| Glisser-déposer | Déposez des fichiers/dossiers dans la zone centrale |
| Parcourir | Cliquez sur la zone de dépôt pour ouvrir le sélecteur |
| Coller | `Ctrl+V` (ou `Cmd+V` sur macOS) avec des chemins de fichiers copiés |

Les fichiers ajoutés s'affichent avec leur **icône** ou **miniature** selon le
type :

- images (png, jpg, gif, webp…) → **miniature réelle**
- vidéos (mp4, mkv…) → icône film
- archives (zip, rar, 7z…) → icône archive
- code (rs, js, py…) → icône code
- documents (pdf, docx, txt…) → icône document

### 2. Sélectionner les destinataires

Dans le panneau droit, cochez les appareils à atteindre.

### 3. Transférer

Cliquez sur **Transférer**. Le bouton reste désactivé tant que vous n'avez pas
des fichiers **et** au moins un appareil coché — le survol indique la raison.

L'application bascule automatiquement sur la page **Transferts** pour suivre la
progression.

## Suivi des transferts

Sur la page Transferts, chaque transfert affiche :

- le **premier fichier** envoyé (+ le nombre de fichiers restants),
- une **barre de progression globale** (pourcentage + octets envoyés/total),
- le **débit** en temps réel (o/s, Ko/s ou Mo/s),
- des **mini-barres par fichier** pour les transferts multi-fichiers.

### Annuler

Cliquez sur le bouton **✕** d'un transfert en cours pour l'annuler. L'annulation
est propre : le récepteur signale un échec (jamais une réception tronquée) et le
fichier partiel est laissé tel quel dans le dossier de destination.

## Réception

- Le récepteur écoute sur le port **58200** dès le démarrage de l'application.
- Les fichiers arrivent dans **`Téléchargements/Toolé/`** (ou `Downloads/Toolé`
  selon la langue du système).
- L'arborescence des dossiers est **conservée** : un dossier envoyé est recréé
  à l'identique sous `Téléchargements/Toolé/`.
- Les fichiers reçus apparaissent dans l'**historique** (page Historique) avec
  l'émetteur et le volume transféré.

## Historique

La page **Historique** liste les transferts **terminés, annulés ou en échec** :

- **Statut** : Terminé (vert), Annulé (gris), Erreur (rouge avec le message).
- **Détails** : nom du premier fichier, appareil pair, taille transférée, heure.
- **Supprimer** une entrée individuelle (icône poubelle sur la ligne).
- **Tout effacer** (bouton en haut à droite) pour vider l'historique — les
  transferts encore en cours sont conservés.

L'historique est **persisté** en local (localStorage, max 200 entrées) : il
survit au redémarrage de l'application.

## Paramètres

La page **Paramètres** permet de personnaliser l'interface :

### Apparence (thème)

- **Suivre le système** : Toolé adopte le thème sombre/clair de l'OS.
- **Sombre** / **Clair** : forcer un thème.

### Luminosité (halo)

- **Lueur** : curseur 0–100 % pour l'intensité du halo d'arrière-plan.
- **Couleur de la lueur** : 8 teintes au choix (rouge, orange, jaune, vert,
  cyan, bleu, violet, rose). La couleur devient aussi la couleur d'accent de
  l'interface.

### À propos

Le bouton **À propos de Toolé** ouvre une modale avec la version et les crédits.

### Mises à jour

Toolé vérifie les mises à jour **au démarrage** (en silence) et à l'ouverture de
ce panneau :

- **Rechercher une mise à jour** : interroge GitHub pour une version plus
  récente.
- Si une nouvelle version existe, un bouton **Installer et redémarrer**
  télécharge puis relance l'application à jour.
- La MAJ automatique est disponible sur **Windows** (via l'installeur
  `setup.exe`) et sur **Linux** (via le paquet `.deb` ou `.rpm` correspondant à
  votre distribution).
- Sur **macOS**, la MAJ automatique est désactivée (binaires non notariés) :
  téléchargez le nouveau `.dmg` depuis le site ou GitHub et remplacez
  l'application.

Sans connexion internet, aucune erreur ne s'affiche : la vérification échoue
silencieusement et le bouton reste disponible.

## Raccourcis clavier

| Raccourci | Action |
|---|---|
| `Ctrl+V` / `Cmd+V` | Coller des chemins de fichiers dans la zone de dépôt |
| Entrée / Espace | Sélectionner l'appareil en surbrillance |

---

> [Sommaire](index.md) · Problème rencontré ? → [Dépannage](depannage.md)
