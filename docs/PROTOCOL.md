# Protocole Réseau — Toolé

## Discovery Packet

Diffusé en UDP broadcast par le sender.

```json
{
  "type": "discovery",
  "device_name": "Nahounou-PC",
  "port": 8080,
  "protocol_version": 1
}
```

---

## Handshake Packet

Envoyé par le receiver après la connexion TCP.

```json
{
  "type": "handshake",
  "device_id": "uuid",
  "protocol_version": 1
}
```

---

## Metadata Packet

Envoyé par le sender avant le transfert.

```json
{
  "packet_type": "metadata",
  "file_id": "uuid",
  "filename": "video.mp4",
  "size": 104857600,
  "chunk_size": 65536,
  "sha256": "..."
}
```

---

## Chunk Packet

Chunk de données avec validation CRC32.

```json
{
  "packet_type": "chunk",
  "file_id": "uuid",
  "chunk_index": 12,
  "crc32": "...",
  "data": "binary"
}
```

---

## Complete Packet

Envoyé après le dernier chunk.

```json
{
  "packet_type": "complete",
  "file_id": "uuid"
}
```
