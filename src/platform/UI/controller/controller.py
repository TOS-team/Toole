from typing import List


class Controller:
    def __init__(self):
        self.current_user = "Gérard_BIT"
        self.local_ip = "192.168.1.10"
        self.connected_users = []
        self.pending_requests = []

    def accept_connection(self, username, ip):
        print(f"✅ Connexion acceptée avec {username} ({ip})")
        self.connected_users.append({"name": username, "ip": ip, "status": "Connected"})

    def decline_connection(self, username):
        print(f"❌ Connexion refusée avec {username}")

    def accept_file_transfer(self, sender, filename, size):
        print(f"✅ Transfert accepté : {filename} ({size}) de {sender}")

    def send_files(self, filepaths: List[str], recipient: str):
        """Méthode appelée quand l'utilisateur clique sur Envoyer"""
        if not filepaths:
            print("⚠️ Aucune fichier à envoyer")
            return

        print(f"✅ Controller : Demande d'envoi de {len(filepaths)} élément(s) vers {recipient}")
        for path in filepaths:
            print(f"   📄 {path}")
        
        # Ici tu pourras plus tard ajouter la logique réelle de transfert