#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <poll.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/socket.h>

#include "app.h"
#include "file_transfert.h"

// Hello le BOP, ici je pose des helpers locaux pour garder app.c lisible
static long long now_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec * 1000LL + ts.tv_nsec / 1000000LL;
}

static void sync_presence_fields(toole_app *app)
{
    app->discovery_ctx.self = app->self;
}

static void reset_clients(toole_app *app)
{
    app->client_count = 0;
    for (size_t i = 0; i < TOOLE_MAX_CLIENTS; i++) app->client_sockets[i] = -1;
}

static int add_client_socket(toole_app *app, int fd)
{
    if (fd < 0) return -1;
    if (app->client_count >= TOOLE_MAX_CLIENTS) {
        network_close(fd);
        return -1;
    }
    app->client_sockets[app->client_count++] = fd;
    return 0;
}

static void remove_client_socket(toole_app *app, size_t index)
{
    if (index >= app->client_count) return;
    // Hello la BOP, ici je log quand un client saute pour aider le debug
    fprintf(stderr, "[MASTER] client fd=%d deconnecte (slot %zu)\n",
            app->client_sockets[index], index);
    network_close(app->client_sockets[index]);
    for (size_t i = index; i + 1 < app->client_count; i++) {
        app->client_sockets[i] = app->client_sockets[i + 1];
    }
    app->client_count--;
}

// là je lance la discovery en thread pour garder une boucle app disponible pour le runtime
static void *discovery_thread_main(void *arg)
{
    toole_app *app = (toole_app *)arg;
    int rc = discovery_multiplex(presence, hear, &app->discovery_ctx);
    return (void *)(intptr_t)rc;
}

// ici je cherche un master dans la liste découverte pour tenter une connexion client
static int find_master_candidate(const toole_app *app, info *master_out)
{
    if (!app || !master_out) return -1;
    // Hello la BOP, on lock la liste des devices avant de la parcourir
    toole_mutex_lock(app->devices_lock);
    for (int i = 0; i < app->device_count; i++) {
        const info *node = &app->devices[i].node_info;
        if (node->r != ROLE_MASTER) continue;
        if (strcmp(node->id, app->self.id) == 0) continue;
        *master_out = *node;
        toole_mutex_unlock(app->devices_lock);
        return 0;
    }
    toole_mutex_unlock(app->devices_lock);
    return -1;
}

static int become_master(toole_app *app)
{
    app->self.r = ROLE_MASTER;
    if (app->self.cluster_id[0] == '\0') {
        snprintf(app->self.cluster_id, sizeof(app->self.cluster_id), "%s", app->self.id);
    }
    snprintf(app->self.master_ip, sizeof(app->self.master_ip), "%s", app->self.ip);
    app->self.master_port = app->self.tcp_port;
    app->current_state = MASTER;
    sync_presence_fields(app);
    return 0;
}

// en mode client on essaie direct le master annoncé; sinon fallback via election/reco
static int step_discovering(toole_app *app)
{
    info master_candidate;
    if (find_master_candidate(app, &master_candidate) == 0) {
        int fd = connect_to(master_candidate.ip, (uint16_t)master_candidate.tcp_port);
        if (fd >= 0) {
            app->control_socket = fd;
            app->self.r = ROLE_CLIENT;
            snprintf(app->self.cluster_id, sizeof(app->self.cluster_id), "%s", master_candidate.cluster_id);
            snprintf(app->self.master_ip, sizeof(app->self.master_ip), "%s", master_candidate.ip);
            app->self.master_port = master_candidate.tcp_port;
            sync_presence_fields(app);
            network_send_hello(app->control_socket, &app->self);
            app->current_state = CLIENT;
            return 0;
        }
    }

    // Hello la BOP, on attend un cycle beacon avant de s'auto-elire pour eviter 2 masters
    if (now_ms() - app->discovering_since_ms < app->beacon_interval_ms + 200) {
        return 0;
    }

    info elected;
    toole_mutex_lock(app->devices_lock);
    int elect_rc = runtime_elect_master_from_devices(app->devices, app->device_count, &app->self, &elected);
    toole_mutex_unlock(app->devices_lock);
    if (elect_rc == 0 &&
        strcmp(elected.id, app->self.id) == 0) {
        return become_master(app);
    }
    return 0;
}

static int step_client(toole_app *app)
{
    if (app->control_socket < 0) {
        app->current_state = DISCOVERING;
        app->discovering_since_ms = now_ms();
        return 0;
    }

    int master_lost = 0;
    int rc = runtime_client_step(app->control_socket, &app->self, 1500, &master_lost);
    if (rc < 0 || master_lost) {
        network_close(app->control_socket);
        app->control_socket = -1;
        app->current_state = ELECTION;
    }
    return 0;
}

static void manage_server_socket(toole_app *app) {
    if (app->server_socket < 0) {
        app->server_socket = init_server_on((uint16_t)app->self.tcp_port);
        if (app->server_socket < 0) return;
    }

    struct pollfd incoming = {
        .fd = app->server_socket,
        .events = POLLIN
    };
    int pr = poll(&incoming, 1, 0);
    if (pr > 0 && (incoming.revents & POLLIN)) {
        int cfd = accept_client(app->server_socket);
        if (cfd >= 0) add_client_socket(app, cfd);
    }
}

// Hello la BOP, ici on traite le trafic reseau entrant pour tout le monde (CLIENT et MASTER)
// ca permet d'accepter les connexions TCP ephemeres de transfert direct
static void process_incoming_traffic(toole_app *app) {
    manage_server_socket(app);

    for (size_t i = 0; i < app->client_count;) {
        int fd = app->client_sockets[i];
        struct pollfd pfd = {
            .fd = fd,
            .events = POLLIN
        };
        int rr = poll(&pfd, 1, 0);
        if (rr <= 0 || !(pfd.revents & POLLIN)) {
            i++;
            continue;
        }

        uint32_t header = 0;
        ssize_t peeked = recv(fd, &header, sizeof(header), MSG_PEEK);
        if (peeked <= 0) {
            remove_client_socket(app, i);
            continue;
        }

        if (peeked == (ssize_t)sizeof(header) && header == NETWORK_CTRL_MAGIC) {
            // Hello la BOP, seuls les masters traitent les messages de controle
            if (app->current_state == MASTER) {
                network_ctrl_msg msg;
                int peer_closed = 0;
                int wr = runtime_wait_control(fd, 0, &msg, &peer_closed);
                if (wr < 0) {
                    remove_client_socket(app, i);
                    continue;
                }
                if (wr == 0) {
                    switch (msg.type) {
                        case NETWORK_MSG_RELAY_REQUEST:
                            if (network_send_relay_response(fd, &app->self, app->self.cluster_id) < 0) {
                                remove_client_socket(app, i);
                                continue;
                            }
                            break;
                        case NETWORK_MSG_HELLO:
                        case NETWORK_MSG_HEARTBEAT:
                            break;
                        default:
                            fprintf(stderr, "[MASTER] message inattendu type=%d\n", msg.type);
                            break;
                    }
                }
                i++;
            } else {
                // Si on est simple client, on refuse/ferme les connexions de controle directes
                remove_client_socket(app, i);
            }
            continue;
        }

        // Si c'est pas du controle, c'est un transfert de fichier !
        if (recv_file(fd, app->receive_dir) < 0) {
            fprintf(stderr, "[FILE] echec reception depuis fd=%d\n", fd);
            remove_client_socket(app, i);
            continue;
        }
        fprintf(stderr, "[FILE] fichier recu dans %s\n", app->receive_dir);
        // On ferme la connexion ephemere apres reception reussie
        remove_client_socket(app, i);
    }
}

// en mode master, on gere les annonces periodiques aux clients
static int step_master(toole_app *app)
{
    long long t = now_ms();
    if (t - app->last_master_announce_ms >= 1000) {
        for (size_t i = 0; i < app->client_count;) {
            if (network_send_master_announce(app->client_sockets[i], &app->self) < 0) {
                remove_client_socket(app, i);
                continue;
            }
            i++;
        }
        app->last_master_announce_ms = t;
    }

    return 0;
}

static int step_election(toole_app *app)
{
    info new_master;
    int new_socket = -1;
    state next = ELECTION;

    toole_mutex_lock(app->devices_lock);
    int rc = runtime_client_failover(app->devices, app->device_count, &app->self, &new_master, &new_socket, &next);
    toole_mutex_unlock(app->devices_lock);
    if (rc < 0) {
        app->current_state = DISCOVERING;
        app->discovering_since_ms = now_ms();
        return 0;
    }

    if (next == MASTER) {
        app->control_socket = -1;
        return become_master(app);
    }

    app->self.r = ROLE_CLIENT;
    snprintf(app->self.master_ip, sizeof(app->self.master_ip), "%s", new_master.ip);
    app->self.master_port = new_master.tcp_port;
    if (new_master.cluster_id[0] != '\0') {
        snprintf(app->self.cluster_id, sizeof(app->self.cluster_id), "%s", new_master.cluster_id);
    }
    sync_presence_fields(app);

    app->control_socket = new_socket;
    if (app->control_socket >= 0) network_send_hello(app->control_socket, &app->self);
    app->current_state = CLIENT;
    return 0;
}

int app_init(toole_app *app, const info *self_template, state initial_state, const char *message)
{
    if (!app || !self_template) return -1;
    memset(app, 0, sizeof(*app));

    // Hello la BOP, initialisation du mutex pour devices[]
    app->devices_lock = toole_mutex_create();
    if (!app->devices_lock) {
        fprintf(stderr, "[APP] echec creation mutex\n");
        return -1;
    }

    app->self = *self_template;
    app->current_state = initial_state;
    app->stop_flag = 0;
    app->beacon_interval_ms = 1000;
    app->loop_tick_ms = 200;
    app->control_socket = -1;
    app->server_socket = -1;
    app->discovering_since_ms = now_ms();
    app->last_master_announce_ms = now_ms();
    reset_clients(app);

    snprintf(app->receive_dir, sizeof(app->receive_dir), "%s", "received");

    snprintf(app->message, sizeof(app->message), "%s", message ? message : "");

    app->discovery_ctx.self = app->self;
    app->discovery_ctx.message = app->message;
    app->discovery_ctx.liste = app->devices;
    app->discovery_ctx.nb = &app->device_count;
    app->discovery_ctx.beacon_interval = app->beacon_interval_ms;
    app->discovery_ctx.stop_flag = &app->stop_flag;
    app->discovery_ctx.devices_lock = app->devices_lock;

    return 0;
}

int app_start(toole_app *app)
{
    if (!app) return -1;
    if (app->discovery_started) return 0;

    if (mkdir(app->receive_dir, 0755) < 0 && errno != EEXIST) {
        perror("[APP] mkdir receive_dir");
        return -1;
    }

    if (start_thread(&app->discovery_thread, discovery_thread_main, app) < 0) {
        return -1;
    }
    app->discovery_started = 1;

    if (app->current_state == MASTER) {
        app->self.r = ROLE_MASTER;
        if (app->self.cluster_id[0] == '\0') snprintf(app->self.cluster_id, sizeof(app->self.cluster_id), "%s", app->self.id);
        snprintf(app->self.master_ip, sizeof(app->self.master_ip), "%s", app->self.ip);
        app->self.master_port = app->self.tcp_port;
        sync_presence_fields(app);
    } else if (app->current_state == DISCOVERING) {
        app->discovering_since_ms = now_ms();
    }
    return 0;
}

// Hello la BOP, ce tick est volontairement simple pour etre pilotable plus tard par une UI Python
int app_tick(toole_app *app)
{
    if (!app) return -1;
    if (app->stop_flag) return 0;

    sync_presence_fields(app);

    // Hello la BOP, on traite le trafic reseau entrant (sockets d'ecoute et clients)
    // pour CLIENT et MASTER afin de gerer le transfert direct
    if (app->current_state == MASTER || app->current_state == CLIENT) {
        process_incoming_traffic(app);
    }

    switch (app->current_state) {
        case DISCOVERING: return step_discovering(app);
        case CLIENT: return step_client(app);
        case MASTER: return step_master(app);
        case ELECTION: return step_election(app);
        case CONNECTING:
        default:
            app->current_state = DISCOVERING;
            return 0;
    }
}

void app_request_stop(toole_app *app)
{
    if (!app) return;
    app->stop_flag = 1;
}

void app_shutdown(toole_app *app)
{
    if (!app) return;

    app_request_stop(app);

    if (app->discovery_started && app->discovery_thread) {
        join_thread(app->discovery_thread);
        destroy_thread(app->discovery_thread);
        app->discovery_thread = NULL;
        app->discovery_started = 0;
    }

    if (app->control_socket >= 0) {
        network_close(app->control_socket);
        app->control_socket = -1;
    }
    if (app->server_socket >= 0) {
        network_close(app->server_socket);
        app->server_socket = -1;
    }
    while (app->client_count > 0) remove_client_socket(app, 0);

    // Hello la BOP, destruction du mutex
    if (app->devices_lock) {
        toole_mutex_destroy(app->devices_lock);
        app->devices_lock = NULL;
    }
}

const char *app_state_name(state s)
{
    switch (s) {
        case DISCOVERING: return "DISCOVERING";
        case CONNECTING: return "CONNECTING";
        case CLIENT: return "CLIENT";
        case MASTER: return "MASTER";
        case ELECTION: return "ELECTION";
        default: return "UNKNOWN";
    }
}

int app_snapshot_devices(const toole_app *app, device *out, size_t cap, size_t *written)
{
    if (!app || !out || cap == 0) return -1;
    // Hello la BOP, on lock la liste des devices avant la copie
    toole_mutex_lock(app->devices_lock);
    size_t count = (size_t)app->device_count;
    if (count > cap) count = cap;
    for (size_t i = 0; i < count; i++) out[i] = app->devices[i];
    if (written) *written = count;
    toole_mutex_unlock(app->devices_lock);
    return 0;
}

int app_connect_to_master(toole_app *app, const char *ip, int tcp_port, const char *cluster_id)
{
    if (!app || !ip || tcp_port <= 0 || tcp_port > 65535) return -1;

    int fd = connect_to(ip, (uint16_t)tcp_port);
    if (fd < 0) return -1;

    if (app->control_socket >= 0) {
        network_close(app->control_socket);
        app->control_socket = -1;
    }

    app->control_socket = fd;
    app->self.r = ROLE_CLIENT;
    snprintf(app->self.master_ip, sizeof(app->self.master_ip), "%s", ip);
    app->self.master_port = tcp_port;
    if (cluster_id && cluster_id[0] != '\0') {
        snprintf(app->self.cluster_id, sizeof(app->self.cluster_id), "%s", cluster_id);
    }
    sync_presence_fields(app);

    if (network_send_hello(app->control_socket, &app->self) < 0) {
        network_close(app->control_socket);
        app->control_socket = -1;
        return -1;
    }

    app->current_state = CLIENT;
    return 0;
}
