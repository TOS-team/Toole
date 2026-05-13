#define _POSIX_C_SOURCE 200809L

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "app.h"

static volatile sig_atomic_t g_stop = 0;

// Hello le BOP, handler Ctrl+C pour arreter proprement
static void on_sigint(int sig)
{
    (void)sig;
    g_stop = 1;
}

static void print_usage(const char *argv0)
{
    printf("Usage: %s [id] [username] [ip] [tcp_port] [master|client]\n", argv0);
}

int main(int argc, char **argv)
{
    info self;
    memset(&self, 0, sizeof(self));

    snprintf(self.id, sizeof(self.id), "%s", (argc > 1) ? argv[1] : "N-001");
    snprintf(self.username, sizeof(self.username), "%s", (argc > 2) ? argv[2] : "gerard");
    snprintf(self.ip, sizeof(self.ip), "%s", (argc > 3) ? argv[3] : "127.0.0.1");
    self.tcp_port = (argc > 4) ? atoi(argv[4]) : SERVER_PORT;
    self.r = ROLE_CLIENT;
    snprintf(self.cluster_id, sizeof(self.cluster_id), "%s", "");

    state initial_state = DISCOVERING;
    if (argc > 5) {
        if (strcmp(argv[5], "master") == 0) {
            initial_state = MASTER;
            self.r = ROLE_MASTER;
            snprintf(self.cluster_id, sizeof(self.cluster_id), "%s", self.id);
        } else if (strcmp(argv[5], "client") != 0) {
            print_usage(argv[0]);
            return 1;
        }
    }

    signal(SIGINT, on_sigint);

    toole_app app;
    if (app_init(&app, &self, initial_state, "auto-discovery on") < 0) {
        fprintf(stderr, "[APP] echec init\n");
        return 1;
    }
    if (app_start(&app) < 0) {
        fprintf(stderr, "[APP] echec start\n");
        app_shutdown(&app);
        return 1;
    }

    // là cette boucle est pensée pour un futur pilotage UI Python:
    // aujourd'hui on loggue l'etat, demain l'UI lira et pilotera via une couche bridge
    long long ticks = 0;
    while (!g_stop) {
        if (app_tick(&app) < 0) {
            fprintf(stderr, "[APP] erreur tick\n");
            break;
        }

        if ((ticks % 10) == 0) {
            printf("[APP] state=%s devices=%d clients=%zu role=%s cluster=%s master=%s:%d\n",
                   app_state_name(app.current_state),
                   app.device_count,
                   app.client_count,
                   (app.self.r == ROLE_MASTER) ? "MASTER" : "CLIENT",
                   app.self.cluster_id,
                   app.self.master_ip,
                   app.self.master_port);
            fflush(stdout);
        }

        struct timespec wait_tick = {
            .tv_sec = app.loop_tick_ms / 1000,
            .tv_nsec = (long)(app.loop_tick_ms % 1000) * 1000000L
        };
        nanosleep(&wait_tick, NULL);
        ticks++;
    }

    app_request_stop(&app);
    app_shutdown(&app);
    return 0;
}
