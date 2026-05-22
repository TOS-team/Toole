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



int presence_socket()
{
    // Creation du socket UDP d'emission; retour negatif si echec.
    int socket_udp;
    socket_udp=socket(AF_INET, SOCK_DGRAM,0);
    if (socket_udp < 0)
    {
        perror("La creation du socket du socket a echoué");
        return -1;
    }


    //la fonction setsockopt() de <sys/socket.h> sera utilisé pour preparer le broadcast , c'est elle qui definit le fonctionement du socket
    int enable= 1;
    if(setsockopt(socket_udp, SOL_SOCKET,SO_BROADCAST,&enable,sizeof(enable))<0)
    {
        perror("setsockopt a echoué");
        close(socket_udp);
        return -1;
    }
    return socket_udp;
}

// Presence est la focntion qui emet les beacon
int presence(int socket_udp, const info *self, const char *message)
{
    if (!self) return -1;
    char beacon[512];
    snprintf(beacon, sizeof(beacon),
             "toole|%s|%s|%s|%d|%d|%s|%s|%d|%s",
             self->id,
             self->username,
             self->ip,
             self->tcp_port,
             (int)self->r,
             self->cluster_id,
             self->master_ip,
             self->master_port,
             message ? message : "");

    //Cette structure definit les adresse et port reseau pour entamer  l'emission de données en UDP
    struct sockaddr_in network_utils={
        .sin_family= AF_INET,
        .sin_port= htons(BEACON_PORT),
        .sin_addr.s_addr= inet_addr("255.255.255.255")
    };
    // Envoie du beacon de presence avec l'ip , le port et le message du TCP
    sendto(socket_udp,beacon,strlen(beacon),0,(struct sockaddr *)&network_utils,sizeof(network_utils));
    return 0;
}
//-------------------------------------------------------------------------

int hear_socket(){
    // Creation du socket UDP d'ecoute pour la fonction hear().
    int socket_udp;
    socket_udp = socket(AF_INET, SOCK_DGRAM, 0);
    if (socket_udp<0){
        perror("La creation du socket a echoué");
        return -1;
    }
    // ici j'autorise la reutilisation d'adresse pour eviter les bind en echec au redemarrage
    int reuse = 1;
    if (setsockopt(socket_udp, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse)) < 0) {
        perror("setsockopt SO_REUSEADDR");
        close(socket_udp);
        return -1;
    }
    // là je met un timeout de lecture pour eviter un blocage infini si utilisé hors poll()
    struct timeval tv = { .tv_sec = 1, .tv_usec = 0 };
    if (setsockopt(socket_udp, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) < 0) {
        perror("setsockopt SO_RCVTIMEO");
        close(socket_udp);
        return -1;
    }
    // creation de la stucture(support pour la transmision)
    struct sockaddr_in network_utils=
        {
        .sin_addr.s_addr = INADDR_ANY,
        .sin_family= AF_INET,
        .sin_port= htons(BEACON_PORT)
    };
    //ici j'attache le socket à la structure en haut
    if(bind (socket_udp, (struct sockaddr *) &network_utils, sizeof( network_utils))< 0)
    {
        perror("bind() a echoué");
        close(socket_udp);
        return -1;
    }
    return socket_udp;
}



// Cette fonction supprime les appareils muets depuis plus de 10 secondes.
void cleaner(device *liste ,int *nb){
    time_t now=time(NULL);
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
//hear ecoute les beacon sur le port d'emmision
void hear(int socket_udp,device *liste,int *nb, const char *self_id)
{
    char buffer[512];
        struct sockaddr_in sender_addr;
        socklen_t size_of = sizeof(sender_addr);
        ssize_t result=recvfrom(socket_udp, buffer, sizeof(buffer)-1, 0,(struct sockaddr *)&sender_addr, &size_of);

        if (result > 0) {
            buffer[result] = '\0';

            //ici je filtre les beacons, pour ne laiser que les beacons avec la signature de toolé
            if (strncmp(buffer, "toole|", 6) == 0) {
                device d;
                int role_tmp = ROLE_CLIENT;
                // Je  parse les beacons recues pour le mettre  dans la structure device que j'ai creé
                int parsed = sscanf(buffer, "toole|%36[^|]|%63[^|]|%15[^|]|%d|%d|%36[^|]|%15[^|]|%d|%127[^\n]",
                                    d.node_info.id,
                                    d.node_info.username,
                                    d.node_info.ip,
                                    &d.node_info.tcp_port,
                                    &role_tmp,
                                    d.node_info.cluster_id,
                                    d.node_info.master_ip,
                                    &d.node_info.master_port,
                                    d.message);
                if (parsed < 8) return;
                if (parsed < 9) d.message[0] = '\0';
                d.node_info.r = (role_tmp == ROLE_MASTER) ? ROLE_MASTER : ROLE_CLIENT;
                d.last_time = time(NULL);

                // Hello la BOP, on filtre les beacons que l'ordinateur s'envoie a lui-meme
                if (self_id && strcmp(d.node_info.id, self_id) == 0) {
                    return;
                }

                /*là pour eviter les doublons de beacons, je verifie la liste, si l'id d'un nouveau becons est deja present dans la liste ,
                 je le suprime et dans le cas contraire , je l'ajoute imediatement
                    */
                int index = -1;
                for (int i = 0; i < *nb; i++) {
                    if (strcmp(liste[i].node_info.id, d.node_info.id) == 0) {
                        index = i;
                        break;
                    }
                }
                if (index != -1) {
                    liste[index] = d;
                }
                else if (*nb < 100) {
                    liste[*nb] = d;
                    (*nb)++;
                }
                else {
                    fprintf(stderr, "[DISCOVERY] liste pleine (100), beacon ignore: %s\n", d.node_info.id);
                }
            }
        }
    }
