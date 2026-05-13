#ifndef BRIDGE_H
#define BRIDGE_H

#include <stddef.h>
#include <stdint.h>

#include "states.h"

#ifdef __cplusplus
extern "C" {
#endif

// Hello le BOP, ici on fige l'API C publique qui sera chargee par Python (ctypes)
// cette phase est volontairement "contrat seulement": pas d'implémentation ici

#define TOOLE_BRIDGE_API_VERSION 1
#define TOOLE_BRIDGE_ERROR_MAX 256
#define TOOLE_BRIDGE_MESSAGE_MAX 128

#if defined(_WIN32)
#define TOOLE_API __declspec(dllexport)
#else
#define TOOLE_API
#endif

typedef enum {
    TOOLE_BRIDGE_OK = 0,
    TOOLE_BRIDGE_ERR_INVALID_ARG = -1,
    TOOLE_BRIDGE_ERR_INIT = -2,
    TOOLE_BRIDGE_ERR_RUNTIME = -3,
    TOOLE_BRIDGE_ERR_IO = -4
} toole_bridge_status;

typedef struct {
    char id[37];
    char username[64];
    char ip[16];
    int tcp_port;
    int start_as_master; // 0 client/discovering, 1 master
    char message[TOOLE_BRIDGE_MESSAGE_MAX];
} toole_bridge_config;

typedef struct {
    char id[37];
    char username[64];
    char ip[16];
    int tcp_port;
    int role; // ROLE_CLIENT / ROLE_MASTER
    char cluster_id[37];
    char master_ip[16];
    int master_port;
} toole_bridge_peer;

typedef struct {
    int state; // enum state
    int role;  // enum role
    int device_count;
    int connected_clients;
    char cluster_id[37];
    char master_ip[16];
    int master_port;
} toole_bridge_snapshot;

typedef struct toole_bridge toole_bridge_t;

TOOLE_API uint32_t toole_bridge_api_version(void);

TOOLE_API toole_bridge_t *toole_bridge_create(void);
TOOLE_API void toole_bridge_destroy(toole_bridge_t *bridge);

TOOLE_API int toole_bridge_init(toole_bridge_t *bridge, const toole_bridge_config *config);
TOOLE_API int toole_bridge_start(toole_bridge_t *bridge);
TOOLE_API int toole_bridge_tick(toole_bridge_t *bridge);
TOOLE_API int toole_bridge_stop(toole_bridge_t *bridge);

TOOLE_API int toole_bridge_get_snapshot(const toole_bridge_t *bridge, toole_bridge_snapshot *out);
TOOLE_API int toole_bridge_get_peers(const toole_bridge_t *bridge, toole_bridge_peer *out, size_t cap, size_t *written);

// Python demandera un transfert via chemin local; le coeur C fait tout le reste
TOOLE_API int toole_bridge_send_file(toole_bridge_t *bridge, const char *path, const char *new_name);

TOOLE_API int toole_bridge_get_last_error(const toole_bridge_t *bridge, char *out, size_t cap);

#ifdef __cplusplus
}
#endif

#endif
