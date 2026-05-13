#ifndef SERVER_RUNTIME_H
#define SERVER_RUNTIME_H

#include "discovery.h"

typedef int (*presence_fn)(
    int socket_udp,
    char *id,
    char *username,
    char *ip,
    int port_tcp,
    char *message
);

typedef void (*hear_fn)(
    int socket_udp,
    device *liste,
    int *nb
);

typedef struct {
    char *id;
    char *username;
    char *ip;
    int port_tcp;
    char *message;

    device *liste;
    int *nb;

    int beacon_interval;
    volatile int *stop_flag;
} context;

int discovery_multiplex(presence_fn presence_cb, hear_fn hear_cb, context *ctx);

#endif
