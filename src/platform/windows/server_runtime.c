/*
    ============================================================================
        VERSION WINDOWS UNIQUEMENT
        Remplacement des APIs Linux/POSIX :
        - poll()        -> WSAPoll()
        - close()       -> closesocket()
        - pthread       -> CreateThread / WaitForSingleObject
        - clock_gettime -> GetTickCount64()
    ============================================================================
*/

#define _WIN32_WINNT 0x0601

#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>

#pragma comment(lib, "Ws2_32.lib")

#include "discovery.h"
#include "network.h"
#include "server_runtime.h"


/*
    ============================================================================
        OUTILS TEMPS WINDOWS
    ============================================================================
*/

/*
    Sous Linux le code utilisait :
        clock_gettime(CLOCK_MONOTONIC)

    Sous Windows on utilise :
        GetTickCount64()

    Cette fonction retourne le temps en millisecondes depuis le démarrage
    du système.
*/
static uint64_t current_time_ms(void)
{
    return GetTickCount64();
}


/*
    ============================================================================
        DISCOVERY MULTIPLEX
    ============================================================================
*/

int discovery_multiplex(presence_fn presence_cb,
                        hear_fn hear_cb,
                        context *ctx)
{
    if (!presence_cb || !hear_cb || !ctx ||
        !ctx->liste || !ctx->nb)
    {
        return -1;
    }

    SOCKET sock_p = presence_socket();
    SOCKET sock_h = hear_socket();

    if (sock_p == INVALID_SOCKET || sock_h == INVALID_SOCKET)
    {
        if (sock_p != INVALID_SOCKET)
            closesocket(sock_p);

        if (sock_h != INVALID_SOCKET)
            closesocket(sock_h);

        return -1;
    }

    WSAPOLLFD wait_beacon;
    wait_beacon.fd = sock_h;
    wait_beacon.events = POLLRDNORM;
    wait_beacon.revents = 0;

    uint64_t last = current_time_ms();

    int interval =
        (ctx->beacon_interval > 0)
        ? ctx->beacon_interval
        : 1000;

    for (;;)
    {
        /*
            Stop demandé
        */
        if (ctx->stop_flag && *ctx->stop_flag)
        {
            break;
        }

        uint64_t now = current_time_ms();

        int elapsed = (int)(now - last);

        int timeout = interval - elapsed;

        if (timeout < 0)
            timeout = 0;

        /*
            Sous Windows :
                poll() -> WSAPoll()
        */
        int r = WSAPoll(&wait_beacon, 1, timeout);

        /*
            Données reçues
        */
        if (r > 0 &&
            (wait_beacon.revents & POLLRDNORM))
        {
            hear_cb((int)sock_h,
                    ctx->liste,
                    ctx->nb);
        }
        /*
            Erreur socket
        */
        else if (r < 0)
        {
            break;
        }

        now = current_time_ms();

        elapsed = (int)(now - last);

        /*
            Temps d'envoyer le beacon
        */
        if (elapsed >= interval)
        {
            presence_cb((int)sock_p,
                        &ctx->self,
                        ctx->message);

            last = now;
        }

        /*
            Nettoyage devices expirés
        */
        cleaner(ctx->liste, ctx->nb);
    }

    closesocket(sock_p);
    closesocket(sock_h);

    return 0;
}


/*
    ============================================================================
        THREAD WINDOWS
    ============================================================================
*/

/*
    Sous Linux :
        pthread_t

    Sous Windows :
        HANDLE
*/

struct thread_runtime
{
    HANDLE handle;
};


/*
    Wrapper pour transmettre la fonction utilisateur
*/
typedef struct
{
    thread_runtime_fn fn;
    void *arg;
} thread_start_data;


/*
    Fonction réelle lancée par CreateThread
*/
static DWORD WINAPI thread_entry(LPVOID param)
{
    thread_start_data *data =
        (thread_start_data *)param;

    if (!data)
        return 0;

    /*
        Appel de la fonction utilisateur
    */
    data->fn(data->arg);

    free(data);

    return 0;
}


/*
    Création thread
*/
int start_thread(thread_runtime_t **out,
                 thread_runtime_fn fn,
                 void *arg)
{
    if (!out || !fn)
        return -1;

    thread_runtime_t *t =
        malloc(sizeof(*t));

    if (!t)
        return -1;

    thread_start_data *data =
        malloc(sizeof(*data));

    if (!data)
    {
        free(t);
        return -1;
    }

    data->fn = fn;
    data->arg = arg;

    t->handle = CreateThread(
        NULL,
        0,
        thread_entry,
        data,
        0,
        NULL
    );

    if (t->handle == NULL)
    {
        free(data);
        free(t);
        return -1;
    }

    *out = t;

    return 0;
}


/*
    Attendre fin thread
*/
int join_thread(thread_runtime_t *thread)
{
    if (!thread)
        return -1;

    DWORD result =
        WaitForSingleObject(
            thread->handle,
            INFINITE
        );

    return (result == WAIT_OBJECT_0)
        ? 0
        : -1;
}


/*
    Détacher thread
*/
int detach_thread(thread_runtime_t *thread)
{
    if (!thread)
        return -1;

    /*
        Windows ne possède pas pthread_detach.

        On ferme simplement le HANDLE.
        Le thread continue d'exister.
    */
    if (!CloseHandle(thread->handle))
        return -1;

    thread->handle = NULL;

    return 0;
}


/*
    Destruction structure thread
*/
void destroy_thread(thread_runtime_t *thread)
{
    if (!thread)
        return;

    if (thread->handle)
    {
        CloseHandle(thread->handle);
    }

    free(thread);
}


/*
    ============================================================================
        HEARTBEAT
    ============================================================================
*/

int runtime_send_heartbeat_once(int socket_tcp,
                                const info *self)
{
    if (!self)
        return -1;

    return network_send_heartbeat(
        socket_tcp,
        self
    );
}


/*
    ============================================================================
        WAIT CONTROL
    ============================================================================
*/

/*
    Attente message TCP avec timeout
*/
int runtime_wait_control(int socket_tcp,
                         int timeout_ms,
                         network_ctrl_msg *msg,
                         int *peer_closed)
{
    if (!msg || timeout_ms < 0)
        return -1;

    if (peer_closed)
        *peer_closed = 0;

    WSAPOLLFD pfd;

    pfd.fd = (SOCKET)socket_tcp;
    pfd.events = POLLRDNORM;
    pfd.revents = 0;

    int r = WSAPoll(&pfd, 1, timeout_ms);

    /*
        Timeout
    */
    if (r == 0)
        return 1;

    /*
        Erreur
    */
    if (r < 0)
        return -1;

    /*
        Rien reçu
    */
    if (!(pfd.revents & POLLRDNORM))
        return 1;

    /*
        Réception message
    */
    if (network_recv_ctrl(socket_tcp,
                          msg,
                          peer_closed) < 0)
    {
        return -1;
    }

    return 0;
}


/*
    ============================================================================
        CLIENT STEP
    ============================================================================
*/

int runtime_client_step(int socket_tcp,
                        const info *self,
                        int timeout_ms,
                        int *master_lost)
{
    if (!self || !master_lost ||
        timeout_ms < 0)
    {
        return -1;
    }

    *master_lost = 0;

    /*
        Envoi heartbeat
    */
    if (runtime_send_heartbeat_once(
            socket_tcp,
            self) < 0)
    {
        *master_lost = 1;
        return -1;
    }

    network_ctrl_msg msg;

    int peer_closed = 0;

    int rc = runtime_wait_control(
        socket_tcp,
        timeout_ms,
        &msg,
        &peer_closed
    );

    /*
        Timeout
    */
    if (rc == 1)
    {
        *master_lost = 1;
        return 0;
    }

    /*
        Erreur réseau
    */
    if (rc < 0)
    {
        if (peer_closed)
            *master_lost = 1;

        return -1;
    }

    /*
        Nouveau master annoncé
    */
    if (msg.type == NETWORK_MSG_MASTER_ANNOUNCE)
    {
        *master_lost = 0;
    }

    return 0;
}


/*
    ============================================================================
        BROADCAST MASTER
    ============================================================================
*/

int runtime_broadcast_master_to_clients(
    const int *client_fds,
    size_t client_count,
    const info *master)
{
    if (!client_fds || !master)
        return -1;

    for (size_t i = 0; i < client_count; i++)
    {
        if (client_fds[i] < 0)
            continue;

        if (network_send_master_announce(
                client_fds[i],
                master) < 0)
        {
            return -1;
        }
    }

    return 0;
}


/*
    ============================================================================
        CANDIDATS ELECTION
    ============================================================================
*/

static int contains_candidate(
    const info *candidates,
    size_t count,
    const char *id)
{
    for (size_t i = 0; i < count; i++)
    {
        if (strcmp(candidates[i].id, id) == 0)
        {
            return 1;
        }
    }

    return 0;
}


/*
    ============================================================================
        ELECTION MASTER
    ============================================================================
*/

int runtime_elect_master_from_devices(
    const device *liste,
    int nb,
    const info *self,
    info *winner)
{
    if (!self || !winner || nb < 0)
        return -1;

    size_t max_candidates =
        (size_t)nb + 1;

    info *candidates =
        malloc(max_candidates * sizeof(info));

    if (!candidates)
        return -1;

    size_t count = 0;

    candidates[count++] = *self;

    for (int i = 0; i < nb; i++)
    {
        const info *node =
            &liste[i].node_info;

        if (node->id[0] == '\0')
            continue;

        if (contains_candidate(
                candidates,
                count,
                node->id))
        {
            continue;
        }

        candidates[count++] = *node;
    }

    int rc = elect_master_smallest_id(
        candidates,
        count,
        winner
    );

    free(candidates);

    return rc;
}


/*
    ============================================================================
        RECONNEXION MASTER
    ============================================================================
*/

int runtime_try_reconnect_from_devices(
    const device *liste,
    int nb,
    const info *self,
    info *master_out,
    int *socket_out)
{
    if (!liste || nb < 0 ||
        !self || !master_out ||
        !socket_out)
    {
        return -1;
    }

    *socket_out = -1;

    for (int i = 0; i < nb; i++)
    {
        const info *node =
            &liste[i].node_info;

        /*
            Connexion directe master
        */
        if (node->master_ip[0] != '\0' &&
            node->master_port > 0)
        {
            int fd_master =
                connect_to(
                    node->master_ip,
                    (uint16_t)node->master_port
                );

            if (fd_master >= 0)
            {
                *master_out = *node;

                snprintf(
                    master_out->ip,
                    sizeof(master_out->ip),
                    "%s",
                    node->master_ip
                );

                master_out->tcp_port =
                    node->master_port;

                master_out->r = ROLE_MASTER;

                *socket_out = fd_master;

                return 0;
            }
        }

        /*
            Fallback voisin relay
        */
        if (node->ip[0] == '\0' ||
            node->tcp_port <= 0)
        {
            continue;
        }

        int fd_neighbor =
            connect_to(
                node->ip,
                (uint16_t)node->tcp_port
            );

        if (fd_neighbor < 0)
            continue;

        info relayed_master;

        char cluster_id[37];

        int relay_rc =
            request_master_via_neighbor(
                fd_neighbor,
                self,
                &relayed_master,
                cluster_id,
                sizeof(cluster_id)
            );

        network_close(fd_neighbor);

        if (relay_rc < 0)
            continue;

        int fd_master =
            connect_to(
                relayed_master.ip,
                (uint16_t)relayed_master.tcp_port
            );

        if (fd_master < 0)
            continue;

        relayed_master.r = ROLE_MASTER;

        if (cluster_id[0] != '\0')
        {
            snprintf(
                relayed_master.cluster_id,
                sizeof(relayed_master.cluster_id),
                "%s",
                cluster_id
            );
        }

        *master_out = relayed_master;

        *socket_out = fd_master;

        return 0;
    }

    return -1;
}


/*
    ============================================================================
        FAILOVER CLIENT
    ============================================================================
*/

int runtime_client_failover(
    const device *liste,
    int nb,
    const info *self,
    info *new_master,
    int *socket_out,
    state *next_state)
{
    if (!self || !new_master ||
        !socket_out || !next_state ||
        nb < 0)
    {
        return -1;
    }

    *socket_out = -1;

    *next_state = ELECTION;

    /*
        Election
    */
    if (runtime_elect_master_from_devices(
            liste,
            nb,
            self,
            new_master) < 0)
    {
        return -1;
    }

    /*
        Je suis le nouveau master
    */
    if (strcmp(new_master->id,
               self->id) == 0)
    {
        *new_master = *self;

        new_master->r = ROLE_MASTER;

        snprintf(
            new_master->master_ip,
            sizeof(new_master->master_ip),
            "%s",
            self->ip
        );

        new_master->master_port =
            self->tcp_port;

        if (new_master->cluster_id[0] == '\0')
        {
            snprintf(
                new_master->cluster_id,
                sizeof(new_master->cluster_id),
                "%s",
                self->id
            );
        }

        *next_state = MASTER;

        return 0;
    }

    /*
        Tentative connexion master élu
    */
    int fd = -1;

    if (new_master->ip[0] != '\0' &&
        new_master->tcp_port > 0)
    {
        fd = connect_to(
            new_master->ip,
            (uint16_t)new_master->tcp_port
        );
    }

    /*
        Fallback
    */
    if (fd < 0)
    {
        if (runtime_try_reconnect_from_devices(
                liste,
                nb,
                self,
                new_master,
                &fd) < 0)
        {
            return -1;
        }
    }

    *socket_out = fd;

    *next_state = CLIENT;

    return 0;
}
