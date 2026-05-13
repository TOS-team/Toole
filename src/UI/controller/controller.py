from typing import List
import sys

# === FIX pour les emojis sur Windows ===
if sys.platform.startswith('win'):
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except:
        pass  # si ça ne marche pas, on continue sans emojis c'est sa la galere de windows


class Controller:
    def __init__(self):
        self.current_user = "Gerard_BIT"
        self.local_ip = "192.168.1.10"
        self.connected_users = []
        self.pending_requests = []

    def accept_connection(self, username, ip):
        print(f"[OK] Connexion acceptée avec {username} ({ip})")
        self.connected_users.append({"name": username, "ip": ip, "status": "Connected"})

    def decline_connection(self, username):
        print(f"[REFUS] Connexion refusée avec {username}")

    def accept_file_transfer(self, sender, filename, size):
        print(f"[OK] Transfert accepté : {filename} ({size}) de {sender}")

    def send_files(self, filepaths: List[str], recipient: str):
        """Méthode appelée quand l'utilisateur clique sur Envoyer"""
        if not filepaths:
            print("[ATTENTION] Aucun fichier à envoyer")
            return

        print(f"[ENVOI] Demande d'envoi de {len(filepaths)} élément(s) vers {recipient}")
        for path in filepaths:
            print(f"   → {path}")