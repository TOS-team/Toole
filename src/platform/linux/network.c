#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>

#define SERVER_PORT 42422
#define BACKLOG 16 //represente le nombre de personne qui peuvent attendre avant que le server  ne les accepts
// Hello la BOP, c'est Gérard, avec cette focntion ,je cree un socket que je vais utilisé tout dans ce fichier network.c

//Prototype des fonctions
int create_socket(void);
int init_server(void);
int accept_client(int socket_tcp);
int denied_client(int socket_tcp);
int connect_to(const char *ip, uint16_t port);


int create_socket(){
    int socket_tcp;
    socket_tcp=socket(AF_INET, SOCK_STREAM,0);
    if (socket_tcp<0)
        {
            perror("La création du socket server a échouer");
            return -1;
        }
    return socket_tcp;
}

// là , c'est le sever TCP  qui est chargé d'etablir la connexion avec les appareils clients
int init_server()
{
    int socket_tcp=create_socket();
    int enable=1;
    if (setsockopt(socket_tcp, SOL_SOCKET, SO_REUSEADDR,&enable, sizeof(enable))<0) {
        perror("setsockopt a echoué");
        close(socket_tcp);
        return -1;
    }
    //je definie la structure comme support de transmision
    struct sockaddr_in network_utils={
        .sin_family=AF_INET,
        .sin_port= htons(SERVER_PORT),
        .sin_addr.s_addr = htonl(INADDR_ANY)
    };

    //là j'attache le socket avec le struture network_utils de type sockaddr_in
    if (bind(socket_tcp,(struct sockaddr *)&network_utils,sizeof(network_utils))<0) {
        perror("Erreur avec bind");
        close(socket_tcp);
        return -1;
    }

    // je met le  server en mode ecoute avec listen()
    if (listen(socket_tcp, BACKLOG)<0) {
        perror("Erreur d'ecoute avec listen()");
        close(socket_tcp);
        return -1;
    }
    return socket_tcp;
}

// Cette focntion est là dans l'eventualité ou un appareil doit se connecter à nous , donc on peut l'accepter
int accept_client(int socket_tcp){
    int client_socket=accept(socket_tcp,NULL,NULL);// je ne renvoie  ici que le socket du client, c'est pour cette raison que l'ip et le port sont à NULL
    if (client_socket < 0) {
            perror("erreur d'acceptation");
            return -1;
        }
        return client_socket;
    // la fonction va retouner -1 , si le  server accept la connexion
}

// là on permet au serveur de refuser une connection entrante,en acceptant et en coupant la connexion imediatement
int denied_client(int socket_tcp){
    int client_socket=accept(socket_tcp,NULL,NULL);// je ne renvoie  ici que le socket du client, c'est pour cette raison que l'ip et le port sont à NULL
    if (client_socket>=0) close(client_socket);
        return 0;
    // la focntion return une valeur >=0 car la connection sera fermé à la fin
}

// ici c'est dans le cas on veut se connecter à un autre server
int connect_to(const char *ip,uint16_t port){
    int socket_tcp=create_socket();
    struct sockaddr_in tunnel;
    memset(&tunnel,0,sizeof(tunnel));// ici j'initialise le structure tunnel à 0 avant d'y mettre quoi que ca soit

    tunnel.sin_family=AF_INET;
    tunnel.sin_port=htons(port);

    // je convetit l'adresse IP qui vient sous forme de chaine de caractère en binaire
    if (inet_pton(AF_INET, ip, &tunnel.sin_addr) != 1) {
            perror("Erreur de conversion de l'adresse IP en binaire");
            close(socket_tcp);
            return -1;
        }

    // initialisation de la connexion avec un server tcp distant
    if (connect(socket_tcp, (struct sockaddr *)&tunnel, sizeof(tunnel)) < 0) {
            perror("Erreur dans la tentative de connexion");
            close(socket_tcp);
            return -1;
        }

    return socket_tcp;
}
