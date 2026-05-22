import ctypes
import os
import sys
from ctypes import POINTER, Structure, c_char, c_char_p, c_int, c_size_t, c_uint32
from typing import List, Optional, Tuple

# ==================== CONSTANTES (bridge.h) ====================
TOOLE_BRIDGE_API_VERSION = 2
TOOLE_BRIDGE_ERROR_MAX = 256
TOOLE_BRIDGE_MESSAGE_MAX = 128

# ==================== STATUTS (bridge.h) ====================
TOOLE_BRIDGE_OK = 0
TOOLE_BRIDGE_ERR_INVALID_ARG = -1
TOOLE_BRIDGE_ERR_INIT = -2
TOOLE_BRIDGE_ERR_RUNTIME = -3
TOOLE_BRIDGE_ERR_IO = -4

# ==================== ÉTATS / RÔLES (states.h) ====================
ROLE_CLIENT = 0
ROLE_MASTER = 1

DISCOVERING = 0
CONNECTING = 1
CLIENT = 2
MASTER = 3
ELECTION = 4


class TooleBridgeConfig(Structure):
    _fields_ = [
        ("id", c_char * 37),
        ("username", c_char * 64),
        ("ip", c_char * 16),
        ("tcp_port", c_int),
        ("start_as_master", c_int),
        ("message", c_char * TOOLE_BRIDGE_MESSAGE_MAX),
    ]


class TooleBridgePeer(Structure):
    _fields_ = [
        ("id", c_char * 37),
        ("username", c_char * 64),
        ("ip", c_char * 16),
        ("tcp_port", c_int),
        ("role", c_int),
        ("cluster_id", c_char * 37),
        ("master_ip", c_char * 16),
        ("master_port", c_int),
    ]


class TooleBridgeSnapshot(Structure):
    _fields_ = [
        ("state", c_int),
        ("role", c_int),
        ("device_count", c_int),
        ("connected_clients", c_int),
        ("cluster_id", c_char * 37),
        ("master_ip", c_char * 16),
        ("master_port", c_int),
    ]


class TooleBridgeTransferStatus(Structure):
    _fields_ = [
        ("active", c_int),
        ("status", c_int),
        ("sent", ctypes.c_uint64),
        ("total", ctypes.c_uint64),
        ("filename", c_char * 256),
    ]


class TooleBridge(ctypes.Structure):
    pass


TooleBridgePtr = POINTER(TooleBridge)


# ==================== CHARGEMENT DE LA LIB ====================
def _default_lib_name() -> str:
    return (
        "toole_bridge.dll" if sys.platform.startswith("win") else "libtoole_bridge.so"
    )


def _resolve_lib_path(lib_path: Optional[str]) -> str:
    if lib_path:
        return lib_path

    lib_name = _default_lib_name()
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    candidates = [
        os.environ.get("TOOLE_BRIDGE_PATH"),
        os.path.join(root_dir, "build", lib_name),
        os.path.join(root_dir, lib_name),
        os.path.join(os.path.dirname(__file__), lib_name),
    ]

    for candidate in candidates:
        if candidate and os.path.isfile(candidate):
            return candidate

    return lib_name


def _configure_api(lib: ctypes.CDLL) -> ctypes.CDLL:
    lib.toole_bridge_api_version.restype = c_uint32

    lib.toole_bridge_create.restype = TooleBridgePtr
    lib.toole_bridge_destroy.argtypes = [TooleBridgePtr]

    lib.toole_bridge_init.argtypes = [TooleBridgePtr, POINTER(TooleBridgeConfig)]
    lib.toole_bridge_init.restype = c_int

    lib.toole_bridge_start.argtypes = [TooleBridgePtr]
    lib.toole_bridge_start.restype = c_int

    lib.toole_bridge_tick.argtypes = [TooleBridgePtr]
    lib.toole_bridge_tick.restype = c_int

    lib.toole_bridge_stop.argtypes = [TooleBridgePtr]
    lib.toole_bridge_stop.restype = c_int

    lib.toole_bridge_get_snapshot.argtypes = [
        TooleBridgePtr,
        POINTER(TooleBridgeSnapshot),
    ]
    lib.toole_bridge_get_snapshot.restype = c_int

    lib.toole_bridge_get_peers.argtypes = [
        TooleBridgePtr,
        POINTER(TooleBridgePeer),
        c_size_t,
        POINTER(c_size_t),
    ]
    lib.toole_bridge_get_peers.restype = c_int

    lib.toole_bridge_connect.argtypes = [TooleBridgePtr, c_char_p, c_int, c_char_p]
    lib.toole_bridge_connect.restype = c_int

    lib.toole_bridge_send_file.argtypes = [
        TooleBridgePtr,
        c_char_p,
        c_char_p,
        c_char_p,
        c_int,
    ]
    lib.toole_bridge_send_file.restype = c_int

    lib.toole_bridge_get_transfer_status.argtypes = [
        TooleBridgePtr,
        POINTER(TooleBridgeTransferStatus),
    ]
    lib.toole_bridge_get_transfer_status.restype = c_int

    lib.toole_bridge_set_receive_dir.argtypes = [TooleBridgePtr, c_char_p]
    lib.toole_bridge_set_receive_dir.restype = c_int

    lib.toole_bridge_get_receive_dir.argtypes = [
        TooleBridgePtr,
        POINTER(c_char),
        c_size_t,
    ]
    lib.toole_bridge_get_receive_dir.restype = c_int

    lib.toole_bridge_get_last_error.argtypes = [
        TooleBridgePtr,
        POINTER(c_char),
        c_size_t,
    ]
    lib.toole_bridge_get_last_error.restype = c_int

    return lib


def load_bridge(lib_path: Optional[str] = None) -> ctypes.CDLL:
    # Charger la lib et déclarer les signatures C
    path = _resolve_lib_path(lib_path)
    lib = _configure_api(ctypes.CDLL(path))
    runtime_version = int(lib.toole_bridge_api_version())
    if runtime_version != TOOLE_BRIDGE_API_VERSION:
        raise RuntimeError(
            f"Version bridge incompatible: Python={TOOLE_BRIDGE_API_VERSION}, C={runtime_version}"
        )
    return lib


# ==================== WRAPPER HAUT NIVEAU ====================
class Bridge:
    def __init__(self, lib_path: Optional[str] = None):
        self._lib = load_bridge(lib_path)
        self._handle = self._lib.toole_bridge_create()
        if not self._handle:
            raise RuntimeError("Impossible de créer le bridge (toole_bridge_create)")

    def close(self):
        if self._handle:
            self._lib.toole_bridge_destroy(self._handle)
            self._handle = None

    def __del__(self):
        self.close()

    def init(self, config: TooleBridgeConfig) -> int:
        return self._lib.toole_bridge_init(self._handle, ctypes.byref(config))

    def start(self) -> int:
        return self._lib.toole_bridge_start(self._handle)

    def tick(self) -> int:
        return self._lib.toole_bridge_tick(self._handle)

    def stop(self) -> int:
        return self._lib.toole_bridge_stop(self._handle)

    def get_snapshot(self) -> Tuple[int, TooleBridgeSnapshot]:
        snap = TooleBridgeSnapshot()
        status = self._lib.toole_bridge_get_snapshot(self._handle, ctypes.byref(snap))
        return status, snap

    def get_peers(self, cap: int = 64) -> Tuple[int, List[TooleBridgePeer]]:
        out = (TooleBridgePeer * cap)()
        written = c_size_t(0)
        status = self._lib.toole_bridge_get_peers(
            self._handle, out, cap, ctypes.byref(written)
        )
        return status, list(out[: written.value])

    def connect(self, ip: str, tcp_port: int, cluster_id: Optional[str] = None) -> int:
        ip_bytes = ip.encode("utf-8")
        cluster_bytes = cluster_id.encode("utf-8") if cluster_id else None
        return self._lib.toole_bridge_connect(
            self._handle, ip_bytes, tcp_port, cluster_bytes
        )

    def send_file(
        self,
        path: str,
        new_name: Optional[str] = None,
        dest_ip: Optional[str] = None,
        dest_port: int = 0,
    ) -> int:
        path_bytes = path.encode("utf-8")
        name_bytes = new_name.encode("utf-8") if new_name else None
        dest_ip_bytes = dest_ip.encode("utf-8") if dest_ip else None
        return self._lib.toole_bridge_send_file(
            self._handle, path_bytes, name_bytes, dest_ip_bytes, dest_port
        )

    def get_transfer_status(self) -> Tuple[int, TooleBridgeTransferStatus]:
        status = TooleBridgeTransferStatus()
        rc = self._lib.toole_bridge_get_transfer_status(
            self._handle, ctypes.byref(status)
        )
        return rc, status

    def set_receive_dir(self, receive_dir: str) -> int:
        return self._lib.toole_bridge_set_receive_dir(
            self._handle, receive_dir.encode("utf-8")
        )

    def get_receive_dir(self) -> Tuple[int, str]:
        buf = ctypes.create_string_buffer(256)
        rc = self._lib.toole_bridge_get_receive_dir(self._handle, buf, 256)
        return rc, buf.value.decode("utf-8", errors="replace")

    def get_last_error(self) -> str:
        buf = ctypes.create_string_buffer(TOOLE_BRIDGE_ERROR_MAX)
        self._lib.toole_bridge_get_last_error(self._handle, buf, TOOLE_BRIDGE_ERROR_MAX)
        return buf.value.decode("utf-8", errors="replace")
