import os
import random
import socket
import sys
from typing import Any, Dict, List, Optional, Tuple

from bridge_ctypes import (
    TOOLE_BRIDGE_OK,
    Bridge,
    TooleBridgeConfig,
)

if sys.platform.startswith("win"):
    # Rien de bloquant ici: on évite juste les warnings de typage liés à reconfigure.
    pass


def _decode_cstr(raw: Any) -> str:
    if isinstance(raw, (bytes, bytearray)):
        return raw.split(b"\x00", 1)[0].decode("utf-8", errors="replace")
    return str(raw)


class Controller:
    def __init__(self):
        self.current_user = self._generate_default_username()
        self.local_ip = self._guess_local_ip()

        self._bridge: Optional[Bridge] = None
        self._running = False
        self._last_error = ""

        self._snapshot: Dict[str, Any] = {}
        self._peers: List[Dict[str, Any]] = []

    def _guess_local_ip(self) -> str:
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.connect(("8.8.8.8", 80))
            ip = s.getsockname()[0]
            s.close()
            return ip
        except Exception:
            return "127.0.0.1"

    def _generate_default_username(self) -> str:
        adjectives = [
            "Swift",
            "Calm",
            "Neo",
            "Bright",
            "Silent",
            "Pixel",
            "Turbo",
            "Lime",
            "Nexus",
        ]
        nouns = [
            "Falcon",
            "Lion",
            "Panda",
            "Fox",
            "Tiger",
            "Orion",
            "Nova",
            "Wave",
            "Spark",
        ]
        return f"{random.choice(adjectives)}_{random.choice(nouns)}_{random.randint(100, 999)}"

    def _make_config(self) -> TooleBridgeConfig:
        cfg = TooleBridgeConfig()
        node_suffix = random.randint(100, 999)
        cfg.id = f"N-UI-{node_suffix}".encode("utf-8")
        cfg.username = self.current_user.encode("utf-8")[:63]
        cfg.ip = self.local_ip.encode("utf-8")[:15]
        cfg.tcp_port = 42422
        cfg.start_as_master = 0
        cfg.message = b"ui-bridge-on"
        return cfg

    def start_backend(self) -> bool:
        if self._running:
            return True

        try:
            self._bridge = Bridge()
            st = self._bridge.init(self._make_config())
            if st != TOOLE_BRIDGE_OK:
                self._last_error = (
                    f"bridge init failed ({st}) : {self._bridge.get_last_error()}"
                )
                return False

            st = self._bridge.start()
            if st != TOOLE_BRIDGE_OK:
                self._last_error = (
                    f"bridge start failed ({st}) : {self._bridge.get_last_error()}"
                )
                return False

            self._running = True
            self._last_error = ""
            return True
        except Exception as e:
            self._last_error = f"backend start exception: {e}"
            self._running = False
            return False

    def stop_backend(self):
        if not self._bridge:
            return

        try:
            self._bridge.stop()
        except Exception:
            pass
        finally:
            self._bridge.close()
            self._bridge = None
            self._running = False

    def tick_backend(self):
        if not self._running or not self._bridge:
            return

        st = self._bridge.tick()
        if st != TOOLE_BRIDGE_OK:
            self._last_error = self._bridge.get_last_error()
            return

        snap_st, snap = self._bridge.get_snapshot()
        if snap_st == TOOLE_BRIDGE_OK:
            self._snapshot = {
                "state": snap.state,
                "role": snap.role,
                "device_count": snap.device_count,
                "connected_clients": snap.connected_clients,
                "cluster_id": _decode_cstr(snap.cluster_id),
                "master_ip": _decode_cstr(snap.master_ip),
                "master_port": snap.master_port,
            }

        peers_st, peers = self._bridge.get_peers(cap=128)
        if peers_st == TOOLE_BRIDGE_OK:
            out: List[Dict[str, Any]] = []
            for p in peers:
                out.append(
                    {
                        "id": _decode_cstr(p.id),
                        "name": _decode_cstr(p.username),
                        "ip": _decode_cstr(p.ip),
                        "tcp_port": p.tcp_port,
                        "role": p.role,
                        "cluster_id": _decode_cstr(p.cluster_id),
                        "master_ip": _decode_cstr(p.master_ip),
                        "master_port": p.master_port,
                    }
                )
            self._peers = out

    def get_runtime_view(self) -> Tuple[Dict[str, Any], List[Dict[str, Any]]]:
        return self._snapshot, self._peers

    def get_last_error(self) -> str:
        return self._last_error

    def connect_to_peer(self, peer: Dict[str, Any]) -> Tuple[bool, str]:
        if not self._running or not self._bridge:
            return False, "Backend non démarré"

        ip = str(peer.get("ip", "")).strip()
        tcp_port = int(peer.get("tcp_port", 0))
        cluster_id = str(peer.get("cluster_id", "")).strip() or None

        if not ip or tcp_port <= 0:
            return False, "Pair invalide"

        st = self._bridge.connect(ip, tcp_port, cluster_id)
        if st != TOOLE_BRIDGE_OK:
            return False, self._bridge.get_last_error()

        self.tick_backend()
        return True, f"Connecté à {ip}:{tcp_port}"

    def can_transfer(self) -> Tuple[bool, str]:
        if not self._running or not self._bridge:
            return False, "Backend non démarré"
        return True, ""

    def send_single_file(self, path: str, recipient: str) -> Tuple[bool, str]:
        if not self._running or not self._bridge:
            return False, "Backend non démarré"

        allowed, reason = self.can_transfer()
        if not allowed:
            return False, reason

        if not path or not os.path.exists(path):
            return False, "Chemin invalide"

        if os.path.isdir(path):
            return False, "Envoi de dossier non supporté pour le moment"

        dest_ip = None
        dest_port = 0
        # Hello la BOP, on parse la chaine recipient "Nom • IP:Port" pour recuperer IP/Port
        if recipient and " • " in recipient and ":" in recipient:
            try:
                parts = recipient.split(" • ")
                addr_part = parts[1]
                dest_ip, port_str = addr_part.split(":")
                dest_port = int(port_str)
            except Exception:
                pass

        name = os.path.basename(path)
        st = self._bridge.send_file(path, name, dest_ip, dest_port)
        if st != TOOLE_BRIDGE_OK:
            return False, self._bridge.get_last_error()

        return True, f"{name} envoyé"

    def send_files(self, filepaths: List[str], recipient: str) -> Tuple[bool, str]:
        if not filepaths:
            return False, "Aucun fichier sélectionné"

        failed: List[str] = []
        sent = 0
        for path in filepaths:
            ok, msg = self.send_single_file(path, recipient)
            if ok:
                sent += 1
            else:
                failed.append(f"{os.path.basename(path)}: {msg}")

        if failed:
            return False, "\n".join(failed)

        return True, f"{sent} fichier(s) envoyé(s)"

    # Compat methods (utilisées par popup.py)
    def accept_connection(self, username, ip):
        print(f"[OK] Connexion acceptée avec {username} ({ip})")

    def decline_connection(self, username):
        print(f"[REFUS] Connexion refusée avec {username}")

    def accept_file_transfer(self, sender, filename, size):
        print(f"[OK] Transfert accepté : {filename} ({size}) de {sender}")
