import sys
from typing import List

if sys.platform.startswith("win"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except:
        pass


class Controller:
    def __init__(self):
        self.current_user = "Gerard_BIT"
        self.local_ip = "192.168.1.10"
        self.connected_users = []

    def accept_connection(self, username, ip):
        print(f"✅ Connexion acceptée avec {username} ({ip})")
        self.connected_users.append({"name": username, "ip": ip})

    def decline_connection(self, username):
        print(f"❌ Connexion refusée avec {username}")

    def send_files(self, filepaths: List[str], recipient: str):
        print(f"🚀 Envoi de {len(filepaths)} élément(s) vers {recipient}")
        for path in filepaths:
            print(f"   → {path}")
