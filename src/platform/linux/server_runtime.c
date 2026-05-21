#define _POSIX_C_SOURCE 200809L

#include <poll.h>
#include <time.h>
#include <errno.h>
#include <unistd.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "discovery.h"
#include "network.h"
#include "server_runtime.h"

// Hello le BOP, ici on implemente le mutex cross-platform pour Linux
// c'est un simple wrapper autour de pthread_mutex_t
// le type est opaque dans le header, ici on revele la structure interne
struct toole_mutex {
    pthread_mutex_t mtx;
};

toole_mutex_t *toole_mutex_create(void)
{
    toole_mutex_t *m = malloc(sizeof(*m));
    if (!m) return NULL;
    if (pthread_mutex_init(&m->mtx, NULL) != 0) {
        free(m);
        return NULL;
    }
    return m;
}

void toole_mutex_destroy(toole_mutex_t *m)
{
    if (!m) return;
    pthread_mutex_destroy(&m->mtx);
    free(m);
}

void toole_mutex_lock(toole_mutex_t *m)
{
    if (m) pthread_mutex_lock(&m->mtx);
}

void toole_mutex_unlock(toole_mutex_t *m)
{
    if (m) pthread_mutex_unlock(&m->mtx);
}


//Hello le BOP , ici dans ce fichier ,je creér des fonctions pour gérer le logique
//concurente  de nos differentes composantes

// cette fonction prend des repères(timespec) dans le temps et renvoit la dureé ecoulé entre ses deux reperes
static int duration(struct timespec a, struct timespec b)
{
    long sec = b.tv_sec - a.tv_sec;
    long nsec = b.tv_nsec - a.tv_nsec;
    if (nsec < 0)
    {
        sec--;
        nsec += 1000000000L;
    }
    int d=(sec * 1000 + nsec / 1000000);
    return d;
}


// là c'est une focntion qui va permettre de multiplexer les fonctions presence et hear de discovery.c
int discovery_multiplex(presence_fn presence_cb, hear_fn hear_cb, context *ctx)
{
    if(!presence_cb ||!hear_cb || !ctx || !ctx->liste ||!ctx->nb) return -1;
    int sock_p= presence_socket();
    int sock_h= hear_socket();
    if (sock_p < 0 || sock_h < 0) {
        if (sock_p >= 0) close(sock_p);
        if (sock_h >= 0) close(sock_h);
        return -1;
    }

struct pollfd wait_beacon = {
    .fd = sock_h,
    .events = POLLIN //pour les données qui entrent
};
// j'enregistre le dernier envoie de beacons
struct timespec last;
clock_gettime(CLOCK_MONOTONIC,&last);

//là c'est une condition ternaire pour definir l'interval de temp entre deux beacons en ms bien sur
int interval= ctx->beacon_interval >0 ? ctx->beacon_interval:1000;

for (;;) {
    if (ctx->stop_flag && *ctx->stop_flag) {
        break;
    }

    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);

    int d=duration(last,now);
    int timeout=interval-d;
    if (timeout<0) timeout=0;

    int r=poll(&wait_beacon,1,timeout);
    if (r>0 && (wait_beacon.revents & POLLIN)) {
        // Hello le BOP, ici on lock avant de toucher à la liste des devices
        // le thread principal lit cette liste en parallele, sans ca c'est data race
        if (ctx->devices_lock) toole_mutex_lock(ctx->devices_lock);
        hear_cb(sock_h, ctx->liste, ctx->nb, ctx->self.id);
        if (ctx->devices_lock) toole_mutex_unlock(ctx->devices_lock);
    }
    else if (r<0 && errno != EINTR) {
        break;
    }

    clock_gettime(CLOCK_MONOTONIC, &now);
    d=duration(last,now);
    if (d >= interval)
    {
        presence_cb(sock_p, &ctx->self, ctx->message);
        last = now;
    }
    // pareil ici, cleaner modifie la liste donc on protege
    if (ctx->devices_lock) toole_mutex_lock(ctx->devices_lock);
    cleaner(ctx->liste, ctx->nb);
    if (ctx->devices_lock) toole_mutex_unlock(ctx->devices_lock);
}
    close(sock_p);
    close(sock_h);
    return 0;
}

// Hello la BOP,cette fonction sera utilisé pour lancé une fonction sur une thread,je compte utilisé pthread
struct thread_runtime{
    pthread_t handle;
};
 int start_thread(thread_runtime_t **out, thread_runtime_fn fn, void *arg)
 {
     if (!out || !fn) return -1;
     thread_runtime_t *t = malloc(sizeof(*t));
     if(!t) return -1;

     int rc=pthread_create(&t->handle,NULL,fn,arg);
     if(rc!=0){
         free(t);
         return -1;
     }
     *out=t;
     return 0;
 }

 int join_thread(thread_runtime_t *thread)
 {
     if (!thread) return -1;
     return (pthread_join(thread->handle, NULL) == 0) ? 0 : -1;
 }

 int detach_thread(thread_runtime_t *thread)
 {
     if (!thread) return -1;
     return (pthread_detach(thread->handle) == 0) ? 0 : -1;
 }

 void destroy_thread(thread_runtime_t *thread)
 {
     free(thread);
 }

// Hello la BOP, ici c'est un wrapper simple pour envoyer un heartbeat(battement de coeur) avec notre protocole
int runtime_send_heartbeat_once(int socket_tcp, const info *self)
{
    if (!self) return -1;
    return network_send_heartbeat(socket_tcp, self);
}

// là on attend un message de controle TCP, avec timeout en milliseconde
// return:
// 0  -> message reçu
// 1  -> timeout (donc potentielle perte master)
// -1 -> erreur reseau
int runtime_wait_control(int socket_tcp, int timeout_ms, network_ctrl_msg *msg, int *peer_closed)
{
    if (!msg || timeout_ms < 0) return -1;
    if (peer_closed) *peer_closed = 0;

    struct pollfd pfd = {
        .fd = socket_tcp,
        .events = POLLIN
    };

    int r = poll(&pfd, 1, timeout_ms);
    if (r == 0) return 1;
    if (r < 0) {
        if (errno == EINTR) return 1;
        return -1;
    }
    if (!(pfd.revents & POLLIN)) return 1;

    if (network_recv_ctrl(socket_tcp, msg, peer_closed) < 0) return -1;
    return 0;
}

// cette fonction représente un "pas" client: heartbeat + attente d'event controle
// elle aide a detecter vite si le master est tombé
int runtime_client_step(int socket_tcp, const info *self, int timeout_ms, int *master_lost)
{
    if (!self || !master_lost || timeout_ms < 0) return -1;
    *master_lost = 0;

    if (runtime_send_heartbeat_once(socket_tcp, self) < 0) {
        *master_lost = 1;
        return -1;
    }

    network_ctrl_msg msg;
    int peer_closed = 0;
    int rc = runtime_wait_control(socket_tcp, timeout_ms, &msg, &peer_closed);
    if (rc == 1) {
        *master_lost = 1;
        return 0;
    }
    if (rc < 0) {
        if (peer_closed) *master_lost = 1;
        return -1;
    }

    if (msg.type == NETWORK_MSG_MASTER_ANNOUNCE) {
        *master_lost = 0;
    }
    return 0;
}

// là on annonce le nouveau master à plusieurs clients connectés
int runtime_broadcast_master_to_clients(const int *client_fds, size_t client_count, const info *master)
{
    if (!client_fds || !master) return -1;
    for (size_t i = 0; i < client_count; i++) {
        if (client_fds[i] < 0) continue;
        if (network_send_master_announce(client_fds[i], master) < 0) {
            return -1;
        }
    }
    return 0;
}

// ce helper evite les doublons de candidats pendant l'election
static int contains_candidate(const info *candidates, size_t count, const char *id)
{
    for (size_t i = 0; i < count; i++) {
        if (strcmp(candidates[i].id, id) == 0) return 1;
    }
    return 0;
}

// ici on repertorie la liste des candidats et on applique la regle: plus petit id gagne
int runtime_elect_master_from_devices(const device *liste, int nb, const info *self, info *winner)
{
    if (!self || !winner || nb < 0) return -1;

    size_t max_candidates = (size_t)nb + 1;
    info *candidates = malloc(max_candidates * sizeof(info));
    if (!candidates) return -1;

    size_t count = 0;
    candidates[count++] = *self;

    for (int i = 0; i < nb; i++) {
        const info *node = &liste[i].node_info;
        if (node->id[0] == '\0') continue;
        if (contains_candidate(candidates, count, node->id)) continue;
        candidates[count++] = *node;
    }

    int rc = elect_master_smallest_id(candidates, count, winner);
    free(candidates);
    return rc;
}

// là on tente d'abord connexion directe au master connu, puis reprise via le voisin relay
int runtime_try_reconnect_from_devices(const device *liste, int nb, const info *self, info *master_out, int *socket_out)
{
    if (!liste || nb < 0 || !self || !master_out || !socket_out) return -1;
    *socket_out = -1;

    for (int i = 0; i < nb; i++) {
        const info *node = &liste[i].node_info;

        if (node->master_ip[0] != '\0' && node->master_port > 0) {
            int fd_master = connect_to(node->master_ip, (uint16_t)node->master_port);
            if (fd_master >= 0) {
                // Hello la BOP, ici on ne copie pas l'identite du voisin, juste le master reseau
                memset(master_out, 0, sizeof(*master_out));
                snprintf(master_out->ip, sizeof(master_out->ip), "%s", node->master_ip);
                master_out->tcp_port = node->master_port;
                master_out->r = ROLE_MASTER;
                if (node->cluster_id[0] != '\0') {
                    snprintf(master_out->cluster_id, sizeof(master_out->cluster_id), "%s", node->cluster_id);
                }
                *socket_out = fd_master;
                return 0;
            }
        }

        if (node->ip[0] == '\0' || node->tcp_port <= 0) continue;
        int fd_neighbor = connect_to(node->ip, (uint16_t)node->tcp_port);
        if (fd_neighbor < 0) continue;

        info relayed_master;
        char cluster_id[37];
        int relay_rc = request_master_via_neighbor(fd_neighbor, self, &relayed_master, cluster_id, sizeof(cluster_id));
        network_close(fd_neighbor);
        if (relay_rc < 0) continue;

        int fd_master = connect_to(relayed_master.ip, (uint16_t)relayed_master.tcp_port);
        if (fd_master < 0) continue;

        relayed_master.r = ROLE_MASTER;
        if (cluster_id[0] != '\0') {
            snprintf(relayed_master.cluster_id, sizeof(relayed_master.cluster_id), "%s", cluster_id);
        }
        *master_out = relayed_master;
        *socket_out = fd_master;
        return 0;
    }

    return -1;
}

// cette fonction est notre routine de failover client:
// 1) passer election
// 2) si je gagne => je deviens master
// 3) sinon je me reconnecte au nouveau master
int runtime_client_failover(const device *liste, int nb, const info *self, info *new_master, int *socket_out, state *next_state)
{
    if (!self || !new_master || !socket_out || !next_state || nb < 0) return -1;
    *socket_out = -1;
    *next_state = ELECTION;

    if (runtime_elect_master_from_devices(liste, nb, self, new_master) < 0) return -1;

    if (strcmp(new_master->id, self->id) == 0) {
        *new_master = *self;
        new_master->r = ROLE_MASTER;
        snprintf(new_master->master_ip, sizeof(new_master->master_ip), "%s", self->ip);
        new_master->master_port = self->tcp_port;
        if (new_master->cluster_id[0] == '\0') {
            snprintf(new_master->cluster_id, sizeof(new_master->cluster_id), "%s", self->id);
        }
        *next_state = MASTER;
        return 0;
    }

    int fd = -1;
    if (new_master->ip[0] != '\0' && new_master->tcp_port > 0) {
        fd = connect_to(new_master->ip, (uint16_t)new_master->tcp_port);
    }
    if (fd < 0) {
        if (runtime_try_reconnect_from_devices(liste, nb, self, new_master, &fd) < 0) {
            return -1;
        }
    }

    *socket_out = fd;
    *next_state = CLIENT;
    return 0;
}
