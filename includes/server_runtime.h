#ifndef SERVER_RUNTIME_H
#define SERVER_RUNTIME_H

#include <stddef.h>

#include "discovery.h"
#include "network.h"

typedef int (*presence_fn)(
    int socket_udp,
    const info *self,
    const char *message
);

typedef void (*hear_fn)(
    int socket_udp,
    device *liste,
    int *nb
);

typedef struct {
    info self;
    const char *message;

    device *liste;
    int *nb;

    int beacon_interval;
    volatile int *stop_flag;
} context;

typedef struct thread_runtime thread_runtime_t;
typedef void *(*thread_runtime_fn)(void *);


int discovery_multiplex(presence_fn presence_cb, hear_fn hear_cb, context *ctx);
int start_thread(thread_runtime_t **out, thread_runtime_fn fn, void *arg);
int join_thread(thread_runtime_t *thread);
int detach_thread(thread_runtime_t *thread);
void destroy_thread(thread_runtime_t *thread);

int runtime_send_heartbeat_once(int socket_tcp, const info *self);
int runtime_wait_control(int socket_tcp, int timeout_ms, network_ctrl_msg *msg, int *peer_closed);
int runtime_client_step(int socket_tcp, const info *self, int timeout_ms, int *master_lost);
int runtime_broadcast_master_to_clients(const int *client_fds, size_t client_count, const info *master);
int runtime_elect_master_from_devices(const device *liste, int nb, const info *self, info *winner);
int runtime_try_reconnect_from_devices(const device *liste, int nb, const info *self, info *master_out, int *socket_out);
int runtime_client_failover(const device *liste, int nb, const info *self, info *new_master, int *socket_out, state *next_state);
#endif
