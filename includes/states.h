#ifndef STATES_H
#define STATES_H

// Hello, le BOP, là on fait un typage partagé des roles , des structures des noeuds et des etats

typedef enum{
    ROLE_CLIENT=0,
    ROLE_MASTER=1
} role;

typedef enum{
    DISCOVERING=0,
    CONNECTING,
    CLIENT,
    MASTER,
    ELECTION
} state;

typedef struct{
    char id[37];
    char username[64];
    char ip[16];
    int tcp_port;
   
    role r;
    char cluster_id[37];
   
    char master_ip[16];
    int master_port;
} info ;
#endif