#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <mswsock.h>

#include "network.h"

/*Implementation   des fonction */

//Fonction qui permet de creer la socket en utilisant le port qu'on passe en parametre
SOCKET create_socket(void){

    SOCKET fd = socket(AF_INET,SOCK_STREAM,0);
    if(fd == INVALID_SOCKET){
        printf("echec de demarrage du socket: %d",WSAGetLastError());
        return INVALID_SOCKET;
    }

    return fd;
}

SOCKET init_server(void){
    SOCKET socket_tcp = create_socket();
    if (socket_tcp == INVALID_SOCKET) {
        return INVALID_SOCKET;
    }

    struct sockaddr_in sock_info;
    sock_info.sin_port = htons(SERVER_PORT);
    sock_info.sin_family = AF_INET;
    sock_info.sin_addr.s_addr = htonl(INADDR_ANY);

    char opt = 1;
    if(setsockopt(socket_tcp, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt))){
        printf("setsockopt a echoué");
        return INVALID_SOCKET;
    }

    if(bind(socket_tcp,(const struct sockaddr*)&sock_info,sizeof(sock_info)) == SOCKET_ERROR ){
        printf("le bind a échoué: %d",WSAGetLastError());
        return INVALID_SOCKET;
    }

    if(listen(socket_tcp,BACKLOG) == SOCKET_ERROR){
        printf("erreur lors de l'ecoute");
        return INVALID_SOCKET;
    }

    return socket_tcp;
}

int accept_client(SOCKET fd){
    SOCKET client_socket=accept(fd,NULL,NULL);// je ne renvoie  ici que le socket du client, c'est pour cette raison que l'ip et le port sont à NULL
    if (client_socket < 0) {
            printf("erreur d'acceptation %d",WSAGetLastError());
            return -1;
        }
        return client_socket;
    // la fonction va retouner -1 , si le  server accept la connexion
}

// là on permet au serveur de refuser une connection entrante,en acceptant et en coupant la connexion imediatement
int denied_client(SOCKET socket_tcp){
    SOCKET client_socket=accept(socket_tcp,NULL,NULL);// je ne renvoie  ici que le socket du client, c'est pour cette raison que l'ip et le port sont à NULL
    if (client_socket>=0) closesocket(client_socket);
        return 0;
    // la focntion return une valeur >=0 car la connection sera fermé à la fin
}


//foncion pour permet  les connections avec le serveur.
SOCKET connect_to(const char * ip,uint16_t port){

    SOCKET fd = socket(AF_INET,SOCK_STREAM,0);
    if(fd == INVALID_SOCKET){
        printf("echec de demarrage du socket: %d",WSAGetLastError());
        return INVALID_SOCKET;
    }

    struct sockaddr_in sock_info;
    sock_info.sin_port = htons(port);
    sock_info.sin_family = AF_INET;
    sock_info.sin_addr.s_addr = inet_addr(ip);

    if(connect(fd,(struct sockaddr*)&sock_info,sizeof(sock_info)) == SOCKET_ERROR){
        printf("connection au serveur a échoué: %d\n",WSAGetLastError());
        closesocket(fd);
        return INVALID_SOCKET;
    }

    return fd;
}
