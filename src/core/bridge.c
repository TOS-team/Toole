#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bridge.h"
#include "app.h"

// Hello le BOP, ici on garde un petit objet bridge pour relier Python et le coeur C
struct toole_bridge {
    toole_app app;
    int initialized;
    int running;
    char last_error[TOOLE_BRIDGE_ERROR_MAX];
};

static void bridge_set_error(toole_bridge_t *bridge, const char *msg)
{
    if (!bridge) return;
    snprintf(bridge->last_error, sizeof(bridge->last_error), "%s", msg ? msg : "");
}

uint32_t toole_bridge_api_version(void)
{
    return TOOLE_BRIDGE_API_VERSION;
}

toole_bridge_t *toole_bridge_create(void)
{
    toole_bridge_t *bridge = calloc(1, sizeof(*bridge));
    if (!bridge) return NULL;
    bridge_set_error(bridge, "");
    return bridge;
}

void toole_bridge_destroy(toole_bridge_t *bridge)
{
    if (!bridge) return;
    if (bridge->running) {
        app_request_stop(&bridge->app);
        app_shutdown(&bridge->app);
    }
    free(bridge);
}

int toole_bridge_init(toole_bridge_t *bridge, const toole_bridge_config *config)
{
    if (!bridge || !config) return TOOLE_BRIDGE_ERR_INVALID_ARG;

    info self;
    memset(&self, 0, sizeof(self));

    snprintf(self.id, sizeof(self.id), "%s", config->id[0] ? config->id : "N-001");
    snprintf(self.username, sizeof(self.username), "%s", config->username[0] ? config->username : "user");
    snprintf(self.ip, sizeof(self.ip), "%s", config->ip[0] ? config->ip : "127.0.0.1");
    self.tcp_port = (config->tcp_port > 0) ? config->tcp_port : SERVER_PORT;
    self.r = ROLE_CLIENT;

    state initial_state = DISCOVERING;
    if (config->start_as_master) {
        initial_state = MASTER;
        self.r = ROLE_MASTER;
        snprintf(self.cluster_id, sizeof(self.cluster_id), "%s", self.id);
    }

    if (app_init(&bridge->app, &self, initial_state, config->message) < 0) {
        bridge_set_error(bridge, "app_init a echoue");
        return TOOLE_BRIDGE_ERR_INIT;
    }

    bridge->initialized = 1;
    bridge->running = 0;
    bridge_set_error(bridge, "");
    return TOOLE_BRIDGE_OK;
}

int toole_bridge_start(toole_bridge_t *bridge)
{
    if (!bridge || !bridge->initialized) return TOOLE_BRIDGE_ERR_INVALID_ARG;
    if (bridge->running) return TOOLE_BRIDGE_OK;

    if (app_start(&bridge->app) < 0) {
        bridge_set_error(bridge, "app_start a echoue");
        return TOOLE_BRIDGE_ERR_RUNTIME;
    }

    bridge->running = 1;
    bridge_set_error(bridge, "");
    return TOOLE_BRIDGE_OK;
}

int toole_bridge_tick(toole_bridge_t *bridge)
{
    if (!bridge || !bridge->initialized || !bridge->running) return TOOLE_BRIDGE_ERR_INVALID_ARG;

    if (app_tick(&bridge->app) < 0) {
        bridge_set_error(bridge, "app_tick a echoue");
        return TOOLE_BRIDGE_ERR_RUNTIME;
    }
    return TOOLE_BRIDGE_OK;
}

int toole_bridge_stop(toole_bridge_t *bridge)
{
    if (!bridge || !bridge->initialized) return TOOLE_BRIDGE_ERR_INVALID_ARG;
    if (!bridge->running) return TOOLE_BRIDGE_OK;

    app_request_stop(&bridge->app);
    app_shutdown(&bridge->app);
    bridge->running = 0;
    bridge_set_error(bridge, "");
    return TOOLE_BRIDGE_OK;
}

int toole_bridge_get_snapshot(const toole_bridge_t *bridge, toole_bridge_snapshot *out)
{
    if (!bridge || !out || !bridge->initialized) return TOOLE_BRIDGE_ERR_INVALID_ARG;

    memset(out, 0, sizeof(*out));
    out->state = bridge->app.current_state;
    out->role = bridge->app.self.r;
    out->device_count = bridge->app.device_count;
    out->connected_clients = (int)bridge->app.client_count;
    snprintf(out->cluster_id, sizeof(out->cluster_id), "%s", bridge->app.self.cluster_id);
    snprintf(out->master_ip, sizeof(out->master_ip), "%s", bridge->app.self.master_ip);
    out->master_port = bridge->app.self.master_port;
    return TOOLE_BRIDGE_OK;
}

int toole_bridge_get_peers(const toole_bridge_t *bridge, toole_bridge_peer *out, size_t cap, size_t *written)
{
    if (!bridge || !out || cap == 0 || !bridge->initialized) return TOOLE_BRIDGE_ERR_INVALID_ARG;

    size_t count = 0;
    device tmp[TOOLE_MAX_DEVICES];
    if (app_snapshot_devices(&bridge->app, tmp, TOOLE_MAX_DEVICES, &count) < 0) {
        return TOOLE_BRIDGE_ERR_RUNTIME;
    }

    size_t to_copy = (count < cap) ? count : cap;
    for (size_t i = 0; i < to_copy; i++) {
        const info *n = &tmp[i].node_info;
        snprintf(out[i].id, sizeof(out[i].id), "%s", n->id);
        snprintf(out[i].username, sizeof(out[i].username), "%s", n->username);
        snprintf(out[i].ip, sizeof(out[i].ip), "%s", n->ip);
        out[i].tcp_port = n->tcp_port;
        out[i].role = n->r;
        snprintf(out[i].cluster_id, sizeof(out[i].cluster_id), "%s", n->cluster_id);
        snprintf(out[i].master_ip, sizeof(out[i].master_ip), "%s", n->master_ip);
        out[i].master_port = n->master_port;
    }

    if (written) *written = to_copy;
    return TOOLE_BRIDGE_OK;
}

int toole_bridge_send_file(toole_bridge_t *bridge, const char *path, const char *new_name)
{
    if (!bridge || !path || !new_name) return TOOLE_BRIDGE_ERR_INVALID_ARG;

    // Hello la BOP, pour l'instant le transfert n'est pas encore branche au runtime
    bridge_set_error(bridge, "send_file pas encore branche au bridge");
    return TOOLE_BRIDGE_ERR_RUNTIME;
}

int toole_bridge_get_last_error(const toole_bridge_t *bridge, char *out, size_t cap)
{
    if (!bridge || !out || cap == 0) return TOOLE_BRIDGE_ERR_INVALID_ARG;
    snprintf(out, cap, "%s", bridge->last_error);
    return TOOLE_BRIDGE_OK;
}
