#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <time.h>
#include <unistd.h>
#include <sys/time.h>
#include <stdio.h>

#include "discovery.h"

#define BEACON_PORT 47272

int presence_socket(void);
int presence(int socket_udp, const info *self, const char *message);
int hear_socket(void);
void cleaner(device *liste, int *nb);
void hear(int socket_udp, device *liste, int *nb, const char *self_id);



int presence_socket(void)
{
    int socket_udp;
    socket_udp = socket(AF_INET, SOCK_DGRAM, 0);
    if (socket_udp < 0) {
        perror("La creation du socket a echoue");
        return -1;
    }

    int enable = 1;
    if (setsockopt(socket_udp, SOL_SOCKET, SO_BROADCAST, &enable, sizeof(enable)) < 0) {
        perror("setsockopt a echoue");
        close(socket_udp);
        return -1;
    }
    return socket_udp;
}

int presence(int socket_udp, const info *self, const char *message)
{
    if (!self) return -1;
    char beacon[512];
    snprintf(beacon, sizeof(beacon),
             "toole|%s|%s|%s|%d|%d|%s|%s|%d|%s",
             self->id, self->username, self->ip,
             self->tcp_port, (int)self->r,
             self->cluster_id, self->master_ip,
             self->master_port,
             message ? message : "");

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(BEACON_PORT),
        .sin_addr.s_addr = inet_addr("255.255.255.255")
    };
    sendto(socket_udp, beacon, strlen(beacon), 0,
           (struct sockaddr *)&addr, sizeof(addr));
    return 0;
}
//-------------------------------------------------------------------------

int hear_socket(void)
{
    int socket_udp;
    socket_udp = socket(AF_INET, SOCK_DGRAM, 0);
    if (socket_udp < 0) {
        perror("La creation du socket a echoue");
        return -1;
    }

    int reuse = 1;
    if (setsockopt(socket_udp, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse)) < 0) {
        perror("setsockopt SO_REUSEADDR");
        close(socket_udp);
        return -1;
    }

    struct timeval tv = { .tv_sec = 1, .tv_usec = 0 };
    if (setsockopt(socket_udp, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) < 0) {
        perror("setsockopt SO_RCVTIMEO");
        close(socket_udp);
        return -1;
    }

    struct sockaddr_in addr = {
        .sin_addr.s_addr = INADDR_ANY,
        .sin_family = AF_INET,
        .sin_port = htons(BEACON_PORT)
    };
    if (bind(socket_udp, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        perror("bind() a echoue");
        close(socket_udp);
        return -1;
    }
    return socket_udp;
}



void cleaner(device *liste, int *nb)
{
    time_t now = time(NULL);
    for (int i = 0; i < *nb; i++) {
        if (difftime(now, liste[i].last_time) > 10) {
            for (int j = i; j < *nb - 1; j++) {
                liste[j] = liste[j + 1];
            }
            (*nb)--;
            i--;
        }
    }
}
void hear(int socket_udp, device *liste, int *nb, const char *self_id)
{
    char buffer[512];
    struct sockaddr_in sender_addr;
    socklen_t size_of = sizeof(sender_addr);
    ssize_t result = recvfrom(socket_udp, buffer, sizeof(buffer) - 1, 0,
                              (struct sockaddr *)&sender_addr, &size_of);

    if (result > 0) {
        buffer[result] = '\0';

        if (strncmp(buffer, "toole|", 6) == 0) {
            device d;
            int role_tmp = ROLE_CLIENT;
            int parsed = sscanf(buffer,
                "toole|%36[^|]|%63[^|]|%15[^|]|%d|%d|%36[^|]|%15[^|]|%d|%127[^\n]",
                d.node_info.id, d.node_info.username, d.node_info.ip,
                &d.node_info.tcp_port, &role_tmp,
                d.node_info.cluster_id, d.node_info.master_ip,
                &d.node_info.master_port, d.message);
            if (parsed < 8) return;
            if (parsed < 9) d.message[0] = '\0';
            d.node_info.r = (role_tmp == ROLE_MASTER) ? ROLE_MASTER : ROLE_CLIENT;
            d.last_time = time(NULL);

            // Hello la BOP, on filtre les beacons que l'ordinateur s'envoie a lui-meme
            if (self_id && strcmp(d.node_info.id, self_id) == 0) {
                return;
            }

            int index = -1;
            for (int i = 0; i < *nb; i++) {
                if (strcmp(liste[i].node_info.id, d.node_info.id) == 0) {
                    index = i;
                    break;
                }
            }
            if (index != -1) {
                liste[index] = d;
            } else if (*nb < 100) {
                liste[*nb] = d;
                (*nb)++;
            } else {
                fprintf(stderr, "[DISCOVERY] liste pleine (100), beacon ignore: %s\n", d.node_info.id);
            }
        }
    }
}
