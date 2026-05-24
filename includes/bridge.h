#ifndef BRIDGE_H
#define BRIDGE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Hello le BOP, ici on fige l'API C publique qui sera chargee par Python (ctypes)
// cette phase est volontairement "contrat seulement": pas d'implémentation ici

#define TOOLE_BRIDGE_API_VERSION 2
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

typedef struct {
    int active;      // 1 transfert en cours, 0 aucun
    int status;      // 0 idle, 1 running, 2 done, -1 failed
    uint64_t sent;
    uint64_t total;
    char filename[256];
} toole_bridge_transfer_status;

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
TOOLE_API int toole_bridge_connect(toole_bridge_t *bridge, const char *ip, int tcp_port, const char *cluster_id);
// Hello le BOP, on ajoute dest_ip et dest_port pour envoyer directement à n'importe quel peer
// si dest_ip est NULL, on tombe sur le control_socket comme avant (retro-compat)
TOOLE_API int toole_bridge_send_file(toole_bridge_t *bridge, const char *path, const char *new_name, const char *dest_ip, int dest_port);
TOOLE_API int toole_bridge_get_transfer_status(const toole_bridge_t *bridge, toole_bridge_transfer_status *out);
TOOLE_API int toole_bridge_set_receive_dir(toole_bridge_t *bridge, const char *receive_dir);
TOOLE_API int toole_bridge_get_receive_dir(const toole_bridge_t *bridge, char *out, size_t cap);

TOOLE_API int toole_bridge_get_last_error(const toole_bridge_t *bridge, char *out, size_t cap);

#ifdef __cplusplus
}
#endif

#endif
