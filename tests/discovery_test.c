#include <poll.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#include "discovery.h"

// Hello le BOP, petit helper pour remplir vite un noeud de test
static void fill_info(info *n, const char *id, const char *user, const char *ip, int tcp_port)
{
    memset(n, 0, sizeof(*n));
    snprintf(n->id, sizeof(n->id), "%s", id);
    snprintf(n->username, sizeof(n->username), "%s", user);
    snprintf(n->ip, sizeof(n->ip), "%s", ip);
    n->tcp_port = tcp_port;
    n->r = ROLE_CLIENT;
    snprintf(n->cluster_id, sizeof(n->cluster_id), "%s", "C-TEST");
    snprintf(n->master_ip, sizeof(n->master_ip), "%s", ip);
    n->master_port = tcp_port;
}

// là on standardise les logs de test pour mieux comprendre chaque etape
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
    printf("[INFO] discovery_test: debut du scenario\n");

    int recv_sock = hear_socket();
    if (recv_sock < 0) {
        return step_ko("creation socket d'ecoute", 1);
    }
    step_ok("creation socket d'ecoute");

    pid_t pid = fork();
    if (pid < 0) {
        close(recv_sock);
        return step_ko("fork sender", 2);
    }
    if (pid > 0) step_ok("fork sender");

    if (pid == 0) {
        sleep(1);
        int send_sock = presence_socket();
        if (send_sock < 0) _exit(10);

        info self;
        fill_info(&self, "D-001", "disc_sender", "127.0.0.1", 44001);
        if (presence(send_sock, &self, "hello-discovery") < 0) {
            close(send_sock);
            _exit(11);
        }
        close(send_sock);
        _exit(0);
    }

    step_ok("attente beacon (poll)");
    struct pollfd pfd = {
        .fd = recv_sock,
        .events = POLLIN
    };

    int pr = poll(&pfd, 1, 4000);
    if (pr <= 0) {
        close(recv_sock);
        waitpid(pid, NULL, 0);
        return step_ko("reception beacon avant timeout", 3);
    }
    step_ok("reception beacon avant timeout");

    device list[100];
    int nb = 0;
    hear(recv_sock, list, &nb, NULL);
    close(recv_sock);
    step_ok("parsing beacon + mise a jour liste devices");

    int status = 0;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        return step_ko("processus sender termine proprement", 4);
    }
    step_ok("processus sender termine proprement");

    int found = 0;
    for (int i = 0; i < nb; i++) {
        if (strcmp(list[i].node_info.id, "D-001") == 0) {
            found = 1;
            break;
        }
    }

    if (!found) {
        return step_ko("presence de D-001 dans la liste", 5);
    }
    step_ok("presence de D-001 dans la liste");

    if (strcmp(list[0].node_info.cluster_id, "C-TEST") != 0) {
        return step_ko("validation cluster_id beacon", 6);
    }
    step_ok("validation cluster_id beacon");

    printf("[OK] discovery test reussi (nb=%d)\n", nb);
    return 0;
}
