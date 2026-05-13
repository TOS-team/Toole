#ifndef APP_H
#define APP_H

#include <stddef.h>

#include "server_runtime.h"

#define TOOLE_MAX_DEVICES 100
#define TOOLE_MAX_CLIENTS 32

typedef struct {
    info self;
    state current_state;

    volatile int stop_flag;
    int beacon_interval_ms;
    int loop_tick_ms;
    char message[128];

    device devices[TOOLE_MAX_DEVICES];
    int device_count;

    context discovery_ctx;
    thread_runtime_t *discovery_thread;
    int discovery_started;

    int control_socket;
    int server_socket;
    int client_sockets[TOOLE_MAX_CLIENTS];
    size_t client_count;

    long long last_master_announce_ms;
} toole_app;

int app_init(toole_app *app, const info *self_template, state initial_state, const char *message);
int app_start(toole_app *app);
int app_tick(toole_app *app);
void app_request_stop(toole_app *app);
void app_shutdown(toole_app *app);
const char *app_state_name(state s);
int app_snapshot_devices(const toole_app *app, device *out, size_t cap, size_t *written);

#endif
