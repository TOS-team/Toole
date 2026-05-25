import os
import random
import socket
import subprocess
import sys
import time
from typing import Any, Dict, List, Optional, Set, Tuple

from bridge_ctypes import (
    CLIENT,
    TOOLE_BRIDGE_OK,
    Bridge,
    TooleBridgeConfig,
)


def _decode_cstr(raw: Any) -> str:
    if isinstance(raw, (bytes, bytearray)):
        return raw.split(b"\x00", 1)[0].decode("utf-8", errors="replace")
    return str(raw)


def _default_downloads_dir() -> str:
    if sys.platform == "win32":
        try:
            import ctypes
            from ctypes import wintypes
            FOLDERID_Downloads = "{374DE290-123F-4565-9164-39C4925E467B}"
            SHGetKnownFolderPath = ctypes.windll.shell32.SHGetKnownFolderPath
            SHGetKnownFolderPath.restype = wintypes.HRESULT
            SHGetKnownFolderPath.argtypes = [
                ctypes.c_char_p, wintypes.DWORD, wintypes.HANDLE,
                ctypes.POINTER(ctypes.c_wchar_p),
            ]
            pPath = ctypes.c_wchar_p()
            if SHGetKnownFolderPath(FOLDERID_Downloads.encode("utf-8"), 0, None, ctypes.byref(pPath)) == 0:
                return pPath.value
        except Exception:
            pass
        return os.path.join(os.path.expanduser("~"), "Downloads")
    try:
        result = subprocess.run(
            ["xdg-user-dir", "DOWNLOAD"],
            capture_output=True, text=True, timeout=2,
        )
        path = result.stdout.strip()
        if path and os.path.isabs(path):
            return path
    except Exception:
        pass
    return os.path.expanduser("~/Downloads")


class Controller:
    def __init__(self):
        self.current_user = self._generate_default_username()
        self.local_ip = self._guess_local_ip()
        self.node_id = f"N-UI-{random.randint(1000, 9999)}"
        self.tcp_port = self._pick_available_tcp_port()

        self._bridge: Optional[Bridge] = None
        self._running = False
        self._last_error = ""

        self._snapshot: Dict[str, Any] = {}
        self._peers: List[Dict[str, Any]] = []
        self._events: List[Dict[str, Any]] = []
        self._received_files: List[Dict[str, Any]] = []
        self._known_peer_ids = set()
        self._known_received_paths = set()
        self._baseline_received_paths: Set[str] = set()
        self.receive_dir = _default_downloads_dir()

    def _guess_local_ip(self) -> str:
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.connect(("8.8.8.8", 80))
            ip = s.getsockname()[0]
            s.close()
            return ip
        except Exception:
            return "127.0.0.1"

    def _pick_available_tcp_port(self) -> int:
        # Hello la BOP, on evite de forcer tous les clients UI sur 42422.
        # Un port libre par instance permet de lancer plusieurs noeuds sur la meme machine.
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.bind(("", 0))
            port = s.getsockname()[1]
            s.close()
            return int(port)
        except Exception:
            return 42422

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
        cfg.id = self.node_id.encode("utf-8")[:36]
        cfg.username = self.current_user.encode("utf-8")[:63]
        cfg.ip = self.local_ip.encode("utf-8")[:15]
        cfg.tcp_port = self.tcp_port
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
                self._bridge.close()
                self._bridge = None
                return False

            st = self._bridge.set_receive_dir(self.receive_dir)
            if st != TOOLE_BRIDGE_OK:
                self._last_error = (
                    f"receive dir failed ({st}) : {self._bridge.get_last_error()}"
                )
                self._bridge.close()
                self._bridge = None
                return False

            st = self._bridge.start()
            if st != TOOLE_BRIDGE_OK:
                self._last_error = (
                    f"bridge start failed ({st}) : {self._bridge.get_last_error()}"
                )
                self._bridge.close()
                self._bridge = None
                return False

            self._snapshot_baseline_received()
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
            self._snapshot = {}
            self._peers = []
            self._known_peer_ids.clear()

    def _push_event(self, kind: str, message: str, **data):
        self._events.append(
            {
                "kind": kind,
                "message": message,
                "time": time.strftime("%H:%M:%S"),
                **data,
            }
        )
        self._events = self._events[-100:]

    def _snapshot_baseline_received(self):
        self._baseline_received_paths = set()
        try:
            for name in os.listdir(self.receive_dir):
                path = os.path.join(self.receive_dir, name)
                if os.path.isfile(path):
                    self._baseline_received_paths.add(os.path.normpath(path))
        except OSError:
            pass

    def _scan_received_files(self):
        os.makedirs(self.receive_dir, exist_ok=True)
        files: List[Dict[str, Any]] = []
        for name in sorted(os.listdir(self.receive_dir)):
            path = os.path.join(self.receive_dir, name)
            if not os.path.isfile(path):
                continue
            if os.path.normpath(path) in self._baseline_received_paths:
                continue
            try:
                st = os.stat(path)
            except OSError:
                continue
            item = {
                "name": name,
                "path": path,
                "size": st.st_size,
                "mtime": st.st_mtime,
                "time": time.strftime("%H:%M:%S", time.localtime(st.st_mtime)),
            }
            files.append(item)
            if path not in self._known_received_paths:
                self._known_received_paths.add(path)
                self._push_event("received", f"Fichier recu: {name}", file=item)
        files.sort(key=lambda item: item["mtime"], reverse=True)
        self._received_files = files

    def tick_backend(self):
        if not self._running or not self._bridge:
            self._scan_received_files()
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
            current_ids = {p["id"] for p in out if p.get("id")}
            for peer in out:
                peer_id = peer.get("id")
                if peer_id and peer_id not in self._known_peer_ids:
                    self._push_event(
                        "peer_joined",
                        f"Pair detecte: {peer.get('name') or peer_id}",
                        peer=peer,
                    )
            for peer_id in self._known_peer_ids - current_ids:
                self._push_event("peer_left", f"Pair perdu: {peer_id}", peer_id=peer_id)
            self._known_peer_ids = current_ids
            self._peers = out

        self._scan_received_files()

    def get_runtime_view(self) -> Tuple[Dict[str, Any], List[Dict[str, Any]]]:
        return self._snapshot, self._peers

    def get_events(self) -> List[Dict[str, Any]]:
        return list(reversed(self._events))

    def get_received_files(self) -> List[Dict[str, Any]]:
        self._scan_received_files()
        return list(self._received_files)

    def get_transfer_status(self) -> Dict[str, Any]:
        if not self._bridge:
            return {"active": 0, "status": 0, "sent": 0, "total": 0, "filename": ""}
        rc, st = self._bridge.get_transfer_status()
        if rc != TOOLE_BRIDGE_OK:
            return {"active": 0, "status": -1, "sent": 0, "total": 0, "filename": ""}
        return {
            "active": st.active,
            "status": st.status,
            "sent": int(st.sent),
            "total": int(st.total),
            "filename": _decode_cstr(st.filename),
        }

    def get_last_error(self) -> str:
        return self._last_error

    def set_receive_dir(self, receive_dir: str) -> Tuple[bool, str]:
        receive_dir = os.path.abspath(receive_dir)
        if self._bridge:
            st = self._bridge.set_receive_dir(receive_dir)
            if st != TOOLE_BRIDGE_OK:
                return False, self._bridge.get_last_error()
        self.receive_dir = receive_dir
        self._known_received_paths.clear()
        self._snapshot_baseline_received()
        try:
            os.makedirs(self.receive_dir, exist_ok=True)
            self._scan_received_files()
        except OSError as e:
            return False, f"Erreur acces dossier: {e}"
        return True, f"Dossier de reception: {receive_dir}"

    def restart_backend(self) -> Tuple[bool, str]:
        self.stop_backend()
        if not self.start_backend():
            return False, self._last_error or "Redemarrage backend impossible"
        self.tick_backend()
        return True, "Backend redemarre"

    def update_username(self, new_name: str) -> Tuple[bool, str]:
        new_name = new_name.strip()
        if not new_name:
            return False, "Nom utilisateur invalide"

        if new_name == self.current_user:
            return True, "Nom utilisateur deja a jour"

        self.current_user = new_name[:63]

        # Le username est fige dans la config C au demarrage.
        # On redemarre donc proprement le bridge pour que les beacons UDP publient le nouveau nom.
        was_running = self._running
        if was_running:
            self.stop_backend()
            if not self.start_backend():
                return False, self._last_error

        return True, "Nom utilisateur mis a jour"

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

    def can_transfer(self, recipient: Optional[str] = None) -> Tuple[bool, str]:
        if not self._running or not self._bridge:
            return False, "Backend non démarré"
        if not self._snapshot:
            return False, "Backend en initialisation"

        state = int(self._snapshot.get("state", -1))
        if recipient and recipient != "Everyone (Broadcast)":
            if not self._targets_for_recipient(recipient):
                return False, "Destinataire introuvable, rafraichis la liste"
            return True, ""

        if not self._peers and state != CLIENT:
            return False, "Aucun destinataire disponible sur le reseau"
        return True, ""

    def _peer_label(self, peer: Dict[str, Any]) -> str:
        return f"{peer.get('name', peer.get('id', '?'))} • {peer.get('ip', '?')}:{peer.get('tcp_port', 0)}"

    def recipient_labels(self) -> List[str]:
        labels = ["Everyone (Broadcast)"]
        labels.extend(self._peer_label(p) for p in self._peers)
        return labels

    def _targets_for_recipient(self, recipient: str) -> List[Dict[str, Any]]:
        if recipient == "Everyone (Broadcast)":
            return list(self._peers)

        for peer in self._peers:
            if self._peer_label(peer) == recipient:
                return [peer]
        return []

    def _send_file_to_target(
        self, path: str, name: str, target: Optional[Dict[str, Any]]
    ) -> Tuple[bool, str]:
        if not self._bridge:
            return False, "Backend non démarré"

        dest_ip = None
        dest_port = 0
        if target:
            dest_ip = str(target.get("ip", "")).strip() or None
            dest_port = int(target.get("tcp_port", 0) or 0)

        st = self._bridge.send_file(path, name, dest_ip, dest_port)
        if st != TOOLE_BRIDGE_OK:
            return False, self._bridge.get_last_error()
        return True, f"{name} envoyé"

    def send_single_file(self, path: str, recipient: str) -> Tuple[bool, str]:
        if not self._running or not self._bridge:
            return False, "Backend non démarré"

        allowed, reason = self.can_transfer(recipient)
        if not allowed:
            return False, reason

        if not path or not os.path.exists(path):
            return False, "Chemin invalide"

        if os.path.isdir(path):
            return False, "Envoi de dossier non supporté pour le moment"

        name = os.path.basename(path)
        targets = self._targets_for_recipient(recipient)

        # Broadcast = envoi direct vers tous les pairs detectes.
        # Si aucun pair n'est detecte, on garde le fallback historique via la connexion master.
        if recipient == "Everyone (Broadcast)":
            if not targets:
                return self._send_file_to_target(path, name, None)

            sent = 0
            failed: List[str] = []
            for target in targets:
                ok, msg = self._send_file_to_target(path, name, target)
                if ok:
                    sent += 1
                else:
                    failed.append(f"{target.get('name', target.get('id', '?'))}: {msg}")

            if failed:
                self._push_event(
                    "send_partial", f"Transfert partiel: {name}", detail=failed[:4]
                )
                return sent > 0, f"{sent}/{len(targets)} destinataire(s). " + "; ".join(
                    failed[:4]
                )
            self._push_event("send_done", f"Fichier envoye: {name}")
            return True, f"{name} envoyé à {sent} destinataire(s)"

        if not targets:
            return False, "Destinataire introuvable, rafraichis la liste"

        ok, msg = self._send_file_to_target(path, name, targets[0])
        self._push_event("send_done" if ok else "send_failed", msg)
        return ok, msg

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


