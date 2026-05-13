<<<<<<< HEAD
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
=======
#ifndef DISCOVERY_H
#define DISCOVERY_H

#include <time.h>
#include "states.h"

typedef struct {
    info node_info;
    char message[128];
    time_t last_time;
} device;

int presence_socket(void);
int hear_socket(void);
int presence(int socket_udp, const info *self, const char *message);
void cleaner(device *liste, int *nb);
void hear(int socket_udp, device *liste, int *nb);
#endif
>>>>>>> 4f2ec6ce32eb8f8092ad46be7fd76220b4f353dd
