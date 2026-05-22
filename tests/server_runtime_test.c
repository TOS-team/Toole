#include <stdio.h>
#include <string.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include "discovery.h"
#include "network.h"
#include "server_runtime.h"
#include "states.h"

// Hello le BOP, helpers pour logs lisibles et dataset de test
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

static void fill_device(device *d, const info *src, const char *msg)
{
    memset(d, 0, sizeof(*d));
    d->node_info = *src;
    snprintf(d->message, sizeof(d->message), "%s", msg ? msg : "");
    d->last_time = time(NULL);
}

// test 1: l'election runtime doit bien choisir le plus petit id
static int test_runtime_election(void)
{
    info self, n1, n2;
    fill_info(&self, "N-300", "10.0.0.3", 45003, ROLE_CLIENT, "C-1");
    fill_info(&n1, "N-200", "10.0.0.2", 45002, ROLE_CLIENT, "C-1");
    fill_info(&n2, "N-100", "10.0.0.1", 45001, ROLE_CLIENT, "C-1");

    device list[2];
    fill_device(&list[0], &n1, "peer-1");
    fill_device(&list[1], &n2, "peer-2");

    info winner;
    if (runtime_elect_master_from_devices(list, 2, &self, &winner) < 0) {
        return step_ko("runtime_elect_master_from_devices", 101);
    }
    if (strcmp(winner.id, "N-100") != 0) {
        return step_ko("validation winner id = N-100", 102);
    }
    step_ok("election runtime (plus petit id)");
    return 0;
}

// test 2: client_step doit detecter la perte master par timeout
static int test_runtime_client_timeout(void)
{
    const uint16_t port = 45100;
    int server_fd = init_server_on(port);
    if (server_fd < 0) return step_ko("init_server_on timeout test", 201);

    pid_t pid = fork();
    if (pid < 0) {
        network_close(server_fd);
        return step_ko("fork timeout test", 202);
    }

    if (pid == 0) {
        int pfd = accept_client(server_fd);
        if (pfd < 0) _exit(10);
        // là on ne renvoie volontairement rien pour forcer le timeout côté client
        sleep(2);
        network_close(pfd);
        network_close(server_fd);
        _exit(0);
    }

    sleep(1);
    info self;
    fill_info(&self, "N-500", "127.0.0.1", port, ROLE_CLIENT, "M-001");

    int cfd = connect_to("127.0.0.1", port);
    if (cfd < 0) return step_ko("connect_to timeout test", 203);

    int master_lost = 0;
    int rc = runtime_client_step(cfd, &self, 300, &master_lost);
    network_close(cfd);

    int status = 0;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        return step_ko("child timeout test termine", 204);
    }

    if (rc < 0) return step_ko("runtime_client_step rc", 205);
    if (master_lost != 1) return step_ko("runtime_client_step master_lost==1", 206);
    step_ok("client_step detecte timeout master");
    return 0;
}

// test 3: failover perdant => reconnexion au nouveau master
static int test_runtime_failover_reconnect(void)
{
    const uint16_t winner_port = 45200;
    int winner_server = init_server_on(winner_port);
    if (winner_server < 0) return step_ko("init_server_on failover test", 301);

    pid_t pid = fork();
    if (pid < 0) {
        network_close(winner_server);
        return step_ko("fork failover test", 302);
    }

    if (pid == 0) {
        int pfd = accept_client(winner_server);
        if (pfd < 0) _exit(20);
        network_close(pfd);
        network_close(winner_server);
        _exit(0);
    }

    sleep(1);

    info self, winner, other;
    fill_info(&self, "N-300", "127.0.0.1", 45203, ROLE_CLIENT, "M-001");
    fill_info(&winner, "N-100", "127.0.0.1", winner_port, ROLE_MASTER, "M-001");
    fill_info(&other, "N-200", "127.0.0.1", 45202, ROLE_CLIENT, "M-001");

    device list[2];
    fill_device(&list[0], &winner, "winner");
    fill_device(&list[1], &other, "other");

    info new_master;
    int socket_out = -1;
    state next_state = ELECTION;

    if (runtime_client_failover(list, 2, &self, &new_master, &socket_out, &next_state) < 0) {
        return step_ko("runtime_client_failover", 303);
    }
    if (next_state != CLIENT) return step_ko("next_state=CLIENT", 304);
    if (strcmp(new_master.id, "N-100") != 0) return step_ko("new_master.id", 305);
    if (socket_out < 0) return step_ko("socket_out reconnecte", 306);
    network_close(socket_out);

    int status = 0;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        return step_ko("child failover test termine", 307);
    }
    step_ok("failover client (perdant) + reconnexion");
    return 0;
}

int main(void)
{
    printf("[INFO] server_runtime_test: debut des scenarios\n");

    int rc = test_runtime_election();
    if (rc != 0) return rc;

    rc = test_runtime_client_timeout();
    if (rc != 0) return rc;

    rc = test_runtime_failover_reconnect();
    if (rc != 0) return rc;

    printf("[OK] server_runtime test reussi\n");
    return 0;
}
