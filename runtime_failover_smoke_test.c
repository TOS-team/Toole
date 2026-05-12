#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <time.h>

#include "states.h"
#include "discovery.h"
#include "network.h"
#include "server_runtime.h"

// Hello la BOP, helper pour preparer vite les infos des noeuds
static void fill_info(info *n, const char *id, const char *ip, int tcp_port, role r, const char *cluster_id)
{
    memset(n, 0, sizeof(*n));
    snprintf(n->id, sizeof(n->id), "%s", id);
    snprintf(n->username, sizeof(n->username), "user_%s", id);
    snprintf(n->ip, sizeof(n->ip), "%s", ip);
    n->tcp_port = tcp_port;
    n->r = r;
    snprintf(n->cluster_id, sizeof(n->cluster_id), "%s", cluster_id);
    snprintf(n->master_ip, sizeof(n->master_ip), "%s", ip);
    n->master_port = tcp_port;
}

static void fill_device(device *d, const info *node, const char *msg)
{
    memset(d, 0, sizeof(*d));
    d->node_info = *node;
    snprintf(d->message, sizeof(d->message), "%s", msg ? msg : "");
    d->last_time = time(NULL);
}

// là on test le cas ou je gagne l'election => passage MASTER sans socket de reco
static int test_self_wins_failover(void)
{
    info self;
    fill_info(&self, "N-001", "127.0.0.1", 43001, ROLE_CLIENT, "N-001");

    info other1, other2;
    fill_info(&other1, "N-100", "127.0.0.1", 43010, ROLE_CLIENT, "N-001");
    fill_info(&other2, "N-200", "127.0.0.1", 43020, ROLE_CLIENT, "N-001");

    device liste[2];
    fill_device(&liste[0], &other1, "peer1");
    fill_device(&liste[1], &other2, "peer2");

    info elected;
    int socket_out = -1;
    state next_state = ELECTION;

    if (runtime_client_failover(liste, 2, &self, &elected, &socket_out, &next_state) < 0) return -1;
    if (next_state != MASTER) return -1;
    if (strcmp(elected.id, "N-001") != 0) return -1;
    if (socket_out != -1) return -1;
    return 0;
}

// ici on test le cas ou je perds l'election => reconnexion au nouveau master
static int test_loser_reconnects(void)
{
    const uint16_t winner_port = 43100;

    // on lance un mini serveur winner pour accepter la reco du client perdant
    pid_t pid = fork();
    if (pid < 0) return -1;

    if (pid == 0) {
        int sfd = init_server_on(winner_port);
        if (sfd < 0) _exit(11);
        int cfd = accept_client(sfd);
        if (cfd < 0) {
            network_close(sfd);
            _exit(12);
        }
        network_close(cfd);
        network_close(sfd);
        _exit(0);
    }

    sleep(1);

    info self;
    fill_info(&self, "N-300", "127.0.0.1", 43103, ROLE_CLIENT, "N-100");

    info winner;
    fill_info(&winner, "N-100", "127.0.0.1", winner_port, ROLE_MASTER, "N-100");

    info other;
    fill_info(&other, "N-200", "127.0.0.1", 43102, ROLE_CLIENT, "N-100");

    device liste[2];
    fill_device(&liste[0], &winner, "winner");
    fill_device(&liste[1], &other, "other");

    info elected;
    int socket_out = -1;
    state next_state = ELECTION;

    int rc = runtime_client_failover(liste, 2, &self, &elected, &socket_out, &next_state);
    if (rc < 0) return -1;
    if (next_state != CLIENT) return -1;
    if (strcmp(elected.id, "N-100") != 0) return -1;
    if (socket_out < 0) return -1;

    network_close(socket_out);

    int status = 0;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) return -1;
    return 0;
}

int main(void)
{
    if (test_self_wins_failover() < 0) {
        fprintf(stderr, "[KO] test_self_wins_failover\n");
        return 1;
    }

    if (test_loser_reconnects() < 0) {
        fprintf(stderr, "[KO] test_loser_reconnects\n");
        return 2;
    }

    printf("[OK] runtime failover smoke test reussi\n");
    return 0;
}
