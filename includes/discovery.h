#ifndef DISCOVERY_H
#define DISCOVERY_H

#include <time.h>

typedef struct {
    char id[37];
    char username[64];
    char ip[16];
    int  port_tcp;
    char message[128];
    time_t last_time;
} device;

int presence_socket(void);
int hear_socket(void);
int presence(int socket_udp, char *id, char *username, char *ip, int port_tcp, char *message);
void cleaner(device *liste, int *nb);
void hear(int socket_udp, device *liste, int *nb);
#endif
