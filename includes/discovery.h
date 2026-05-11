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
