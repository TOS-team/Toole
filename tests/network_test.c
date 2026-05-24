#include <stdio.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#include "network.h"

// Hello le BOP, helper pour fabriquer vite des noeuds de test
static void fill_info(info *n, const char *id, const char *ip, int port, role r, const char *cluster)
{
    memset(n, 0, sizeof(*n));
    snprintf(n->id, sizeof(n->id), "%s", id);
    snprintf(n->username, sizeof(n->username), "user_%s", id);
    snprintf(n->ip, sizeof(n->ip), "%s", ip);
    n->tcp_port = port;
    n->r = r;
    snprintf(n->cluster_id, sizeof(n->cluster_id), "%s", cluster);
    snprintf(n->master_ip, sizeof(n->master_ip), "%s", ip);
    n->master_port = port;
}

// Hello le BOP, logs detaillés pour suivre le scenario reseau sans ambiguite
static void step_ok(const char *step)
{
    printf("[STEP][OK] %s\n", step);
    fflush(stdout);
}

static int step_ko(const char *step, int code)
{
    fprintf(stderr, "[STEP][KO] %s (code=%d)\n", step, code);
    return code;
}

int main(void)
{
    printf("[INFO] network_test: debut du scenario\n");

    const uint16_t port = 44100;
    int server_fd = init_server_on(port);
    if (server_fd < 0) {
        return step_ko("init serveur TCP", 1);
    }
    step_ok("init serveur TCP");

    pid_t pid = fork();
    if (pid < 0) {
        network_close(server_fd);
        return step_ko("fork client", 2);
    }
    if (pid > 0) step_ok("fork client");

    if (pid == 0) {
        sleep(1);
        info self;
        fill_info(&self, "N-300", "127.0.0.1", port, ROLE_CLIENT, "M-001");

        int cfd = connect_to("127.0.0.1", port);
        if (cfd < 0) _exit(10);

        if (network_send_hello(cfd, &self) < 0) _exit(11);
        if (network_send_heartbeat(cfd, &self) < 0) _exit(12);

        info master_out;
        char cluster_out[37];
        if (request_master_via_neighbor(cfd, &self, &master_out, cluster_out, sizeof(cluster_out)) < 0) _exit(13);

        info cands[3];
        fill_info(&cands[0], "N-300", "10.0.0.3", 42422, ROLE_CLIENT, "X");
        fill_info(&cands[1], "N-100", "10.0.0.1", 42422, ROLE_CLIENT, "X");
        fill_info(&cands[2], "N-200", "10.0.0.2", 42422, ROLE_CLIENT, "X");

        info winner;
        if (elect_master_smallest_id(cands, 3, &winner) < 0) _exit(14);
        if (strcmp(winner.id, "N-100") != 0) _exit(15);

        network_close(cfd);
        _exit(0);
    }

    int peer_fd = accept_client(server_fd);
    if (peer_fd < 0) {
        network_close(server_fd);
        return step_ko("accept client", 3);
    }
    step_ok("accept client");

    network_ctrl_msg m;
    int peer_closed = 0;

    if (network_recv_ctrl(peer_fd, &m, &peer_closed) < 0 || m.type != NETWORK_MSG_HELLO) {
        return step_ko("reception HELLO", 4);
    }
    if (strcmp(m.sender.id, "N-300") != 0) {
        return step_ko("validation sender HELLO", 5);
    }
    step_ok("reception et validation HELLO");

    if (network_recv_ctrl(peer_fd, &m, &peer_closed) < 0 || m.type != NETWORK_MSG_HEARTBEAT) {
        return step_ko("reception HEARTBEAT", 6);
    }
    step_ok("reception HEARTBEAT");

    if (network_recv_ctrl(peer_fd, &m, &peer_closed) < 0 || m.type != NETWORK_MSG_RELAY_REQUEST) {
        return step_ko("reception RELAY_REQUEST", 7);
    }
    step_ok("reception RELAY_REQUEST");

    info master;
    fill_info(&master, "M-001", "127.0.0.1", port, ROLE_MASTER, "M-001");
    if (network_send_relay_response(peer_fd, &master, "M-001") < 0) {
        return step_ko("envoi RELAY_RESPONSE", 8);
    }
    step_ok("envoi RELAY_RESPONSE");

    network_close(peer_fd);
    network_close(server_fd);

    int status = 0;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        return step_ko("processus client termine proprement", 9);
    }
    step_ok("processus client termine proprement");

    printf("[OK] network test reussi\n");
    return 0;
}
