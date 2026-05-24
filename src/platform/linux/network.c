#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <errno.h>
#include <sys/time.h>


#include "network.h"

// Hello la BOP, c'est Gérard, avec cette focntion ,je cree un socket que je vais utilisé tout dans ce fichier network.c

//Prototype des fonctions
int create_socket(void);
int init_server(void);
int accept_client(int socket_tcp);
int reject_client(int socket_tcp);
int connect_to(const char *ip, uint16_t port);

// cette creé juste des sockets
int create_socket(void) {
    int socket_tcp;
    socket_tcp = socket(AF_INET, SOCK_STREAM, 0);
    if (socket_tcp < 0) {
        perror("La creation du socket server a echoue");
        return -1;
    }
    return socket_tcp;
}

// là , c'est le sever TCP  qui est chargé d'etablir la connexion avec les appareils clients
int init_server(void)
{
    return init_server_on(SERVER_PORT);
}

// là, c'est la version paramétrable du serveur TCP
int init_server_on(uint16_t port)
{
    int socket_tcp = create_socket();
    if (socket_tcp < 0) return -1;

    int enable = 1;
    if (setsockopt(socket_tcp, SOL_SOCKET, SO_REUSEADDR, &enable, sizeof(enable)) < 0) {
        perror("setsockopt a echoué");
        close(socket_tcp);
        return -1;
    }

    struct sockaddr_in network_utils = {
        .sin_family = AF_INET,
        .sin_port = htons(port),
        .sin_addr.s_addr = htonl(INADDR_ANY)
    };

    if (bind(socket_tcp, (struct sockaddr *)&network_utils, sizeof(network_utils)) < 0) {
        perror("Erreur avec bind");
        close(socket_tcp);
        return -1;
    }

    if (listen(socket_tcp, BACKLOG) < 0) {
        perror("Erreur d'ecoute avec listen()");
        close(socket_tcp);
        return -1;
    }

    return socket_tcp;
}

// Cette focntion est là dans l'eventualité ou un appareil doit se connecter à nous , donc on peut l'accepter
int accept_client(int socket_tcp) {
    int client_socket = accept(socket_tcp, NULL, NULL);
    if (client_socket < 0) {
        perror("erreur d'acceptation");
        return -1;
    }
    return client_socket;
    // la fonction va retouner -1 , si le  server accept la connexion
}

// là on permet au serveur de refuser une connection entrante,en acceptant et en coupant la connexion imediatement
int reject_client(int socket_tcp) {
    int client_socket = accept(socket_tcp, NULL, NULL);
    if (client_socket >= 0) {
        fprintf(stderr, "[NETWORK] connexion refusee (fd=%d)\n", client_socket);
        close(client_socket);
    }
    return 0;
}

// ici c'est dans le cas on veut se connecter à un autre server
int connect_to(const char *ip, uint16_t port) {
    int socket_tcp = create_socket();
    struct sockaddr_in tunnel;
    memset(&tunnel, 0, sizeof(tunnel));

    tunnel.sin_family = AF_INET;
    tunnel.sin_port = htons(port);

    if (inet_pton(AF_INET, ip, &tunnel.sin_addr) != 1) {
        perror("Erreur de conversion de l'adresse IP en binaire");
        close(socket_tcp);
        return -1;
    }

    if (connect(socket_tcp, (struct sockaddr *)&tunnel, sizeof(tunnel)) < 0) {
        perror("Erreur dans la tentative de connexion");
        close(socket_tcp);
        return -1;
    }

    return socket_tcp;
}


// Hello la BOP, ici je pose des helpers privés pour fiabiliser les I/O TCP
// read_n_status:
// 0  -> succes
// 1  -> pair fermé (recv == 0)
// -1 -> erreur systeme
static int write_n(int socket_tcp, const void *buffer, size_t n)
{
    const unsigned char *p = (const unsigned char *)buffer;
    size_t sent = 0;

    while (sent < n) {
        ssize_t w = send(socket_tcp, p + sent, n - sent, 0);
        if (w < 0) {
            if (errno == EINTR) continue;
            perror("[TCP] send");
            return -1;
        }
        if (w == 0) return -1;
        sent += (size_t)w;
    }
    return 0;
}

static int read_n_status(int socket_tcp, void *buffer, size_t n)
{
    unsigned char *p = (unsigned char *)buffer;
    size_t recvv = 0;

    while (recvv < n) {
        ssize_t r = recv(socket_tcp, p + recvv, n - recvv, 0);
        if (r < 0) {
            if (errno == EINTR) continue;
            perror("[TCP] recv");
            return -1;
        }
        if (r == 0) return 1;
        recvv += (size_t)r;
    }
    return 0;
}

// ici je definie les timeouts de lecture/ecriture pour mieux detecter les pannes reseau
int set_socket_timeouts(int socket_tcp, int recv_ms, int send_ms)
{
    if (recv_ms < 0 || send_ms < 0) return -1;

    struct timeval recv_tv = {
        .tv_sec = recv_ms / 1000,
        .tv_usec = (recv_ms % 1000) * 1000
    };
    struct timeval send_tv = {
        .tv_sec = send_ms / 1000,
        .tv_usec = (send_ms % 1000) * 1000
    };

    if (setsockopt(socket_tcp, SOL_SOCKET, SO_RCVTIMEO, &recv_tv, sizeof(recv_tv)) < 0) {
        perror("[TCP] SO_RCVTIMEO");
        return -1;
    }
    if (setsockopt(socket_tcp, SOL_SOCKET, SO_SNDTIMEO, &send_tv, sizeof(send_tv)) < 0) {
        perror("[TCP] SO_SNDTIMEO");
        return -1;
    }
    return 0;
}

// cette fonction envoie une structure complete sans perte partielle
int network_send_struct(int socket_tcp, const void *data, size_t size)
{
    if (!data || size == 0) return -1;
    return write_n(socket_tcp, data, size);
}

// cette fonction recoit une structure complete et dit aussi si le pair est fermé
int network_recv_struct(int socket_tcp, void *data, size_t size, int *peer_closed)
{
    if (!data || size == 0) return -1;
    if (peer_closed) *peer_closed = 0;

    int status = read_n_status(socket_tcp, data, size);
    if (status == 1) {
        if (peer_closed) *peer_closed = 1;
        return -1;
    }
    return (status == 0) ? 0 : -1;
}

// là je construis un message de controle commun pour HELLO, heartbeat, election, etc
int network_make_ctrl_msg(network_ctrl_msg *out, network_msg_type type, const info *sender, const char *payload)
{
    if (!out || !sender) return -1;
    if (type < NETWORK_MSG_HELLO || type > NETWORK_MSG_ELECTION_RESULT) return -1;

    memset(out, 0, sizeof(*out));
    out->magic = NETWORK_CTRL_MAGIC;
    out->version = NETWORK_CTRL_VERSION;
    out->type = (uint16_t)type;
    out->sender = *sender;

    if (payload) {
        snprintf(out->payload, sizeof(out->payload), "%s", payload);
    }
    return 0;
}

int network_send_ctrl(int socket_tcp, const network_ctrl_msg *msg)
{
    if (!msg) return -1;
    return network_send_struct(socket_tcp, msg, sizeof(*msg));
}

int network_recv_ctrl(int socket_tcp, network_ctrl_msg *msg, int *peer_closed)
{
    if (!msg) return -1;

    if (network_recv_struct(socket_tcp, msg, sizeof(*msg), peer_closed) < 0) {
        return -1;
    }

    if (msg->magic != NETWORK_CTRL_MAGIC) return -1;
    if (msg->version != NETWORK_CTRL_VERSION) return -1;
    if (msg->type < NETWORK_MSG_HELLO || msg->type > NETWORK_MSG_ELECTION_RESULT) return -1;
    return 0;
}

// Hello la BOP, ici ce sont des wrappers courts pour simplifier la suite
int network_send_hello(int socket_tcp, const info *self)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_HELLO, self, "HELLO") < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

int network_send_heartbeat(int socket_tcp, const info *self)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_HEARTBEAT, self, "HB") < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

int network_send_master_announce(int socket_tcp, const info *master)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_MASTER_ANNOUNCE, master, master->cluster_id) < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

int network_send_election_vote(int socket_tcp, const info *candidate)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_ELECTION_VOTE, candidate, candidate->id) < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

int network_send_election_result(int socket_tcp, const info *winner)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_ELECTION_RESULT, winner, winner->id) < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

int network_send_relay_request(int socket_tcp, const info *self)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_RELAY_REQUEST, self, "REQ_MASTER") < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

int network_send_relay_response(int socket_tcp, const info *master, const char *cluster_id)
{
    network_ctrl_msg msg;
    if (network_make_ctrl_msg(&msg, NETWORK_MSG_RELAY_RESPONSE, master, cluster_id ? cluster_id : "") < 0) return -1;
    return network_send_ctrl(socket_tcp, &msg);
}

// ici un client demande a un voisin les infos du master et du cluster
int request_master_via_neighbor(int socket_tcp, const info *self, info *master_out, char *cluster_id_out, size_t cluster_id_len)
{
    if (!self || !master_out || !cluster_id_out || cluster_id_len == 0) return -1;

    if (network_send_relay_request(socket_tcp, self) < 0) return -1;

    network_ctrl_msg response;
    int peer_closed = 0;
    if (network_recv_ctrl(socket_tcp, &response, &peer_closed) < 0) return -1;
    if (response.type != NETWORK_MSG_RELAY_RESPONSE) return -1;

    *master_out = response.sender;
    snprintf(cluster_id_out, cluster_id_len, "%s", response.payload);
    return 0;
}

// cette fonction applique une regle deterministe: le plus petit id gagne l'election
int elect_master_smallest_id(const info *candidates, size_t count, info *winner)
{
    if (!candidates || count == 0 || !winner) return -1;

    size_t best = 0;
    for (size_t i = 1; i < count; i++) {
        if (strcmp(candidates[i].id, candidates[best].id) < 0) {
            best = i;
        }
    }

    *winner = candidates[best];
    return 0;
}

// fermeture propre d'un socket tcp
void network_close(int socket_tcp)
{
    if (socket_tcp >= 0) close(socket_tcp);
}
