#ifndef NETWORK_H
#define NETWORK_H

#include <stddef.h>
#include <stdint.h>

#include "states.h"

#define SERVER_PORT 42422
#define BACKLOG 16
#define NETWORK_CTRL_MAGIC 0x544F4F4CUL
#define NETWORK_CTRL_VERSION 1
#define NETWORK_CTRL_PAYLOAD_MAX 128

typedef enum {
    NETWORK_MSG_HELLO = 1,
    NETWORK_MSG_HEARTBEAT = 2,
    NETWORK_MSG_MASTER_ANNOUNCE = 3,
    NETWORK_MSG_RELAY_REQUEST = 4,
    NETWORK_MSG_RELAY_RESPONSE = 5,
    NETWORK_MSG_ELECTION_VOTE = 6,
    NETWORK_MSG_ELECTION_RESULT = 7
} network_msg_type;

typedef struct {
    uint32_t magic;
    uint16_t version;
    uint16_t type;
    info sender;
    char payload[NETWORK_CTRL_PAYLOAD_MAX];
} network_ctrl_msg;

int create_socket(void);
int init_server(void);
int init_server_on(uint16_t port);
int accept_client(int socket_tcp);
int denied_client(int socket_tcp);
int connect_to(const char *ip, uint16_t port);
int set_socket_timeouts(int socket_tcp, int recv_ms, int send_ms);

int network_send_struct(int socket_tcp, const void *data, size_t size);
int network_recv_struct(int socket_tcp, void *data, size_t size, int *peer_closed);

int network_make_ctrl_msg(network_ctrl_msg *out, network_msg_type type, const info *sender, const char *payload);
int network_send_ctrl(int socket_tcp, const network_ctrl_msg *msg);
int network_recv_ctrl(int socket_tcp, network_ctrl_msg *msg, int *peer_closed);

int network_send_hello(int socket_tcp, const info *self);
int network_send_heartbeat(int socket_tcp, const info *self);
int network_send_master_announce(int socket_tcp, const info *master);
int network_send_election_vote(int socket_tcp, const info *candidate);
int network_send_election_result(int socket_tcp, const info *winner);
int network_send_relay_request(int socket_tcp, const info *self);
int network_send_relay_response(int socket_tcp, const info *master, const char *cluster_id);
int request_master_via_neighbor(int socket_tcp, const info *self, info *master_out, char *cluster_id_out, size_t cluster_id_len);

int elect_master_smallest_id(const info *candidates, size_t count, info *winner);
void network_close(int socket_tcp);

#endif
