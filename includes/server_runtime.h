#ifndef SERVER_RUNTIME_H
#define SERVER_RUNTIME_H

#include "discovery.h"

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
#endif
