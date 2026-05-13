#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <mswsock.h>

#define SERVER_PORT 42422
#define BACKLOG 16

/*Implementation des fonction */

//Fonction qui permet de creer la socket en utilisant le port qu'on passe en parametre 
SOCKET init_tcp(int port){

    SOCKET fd = socket(AF_INET,SOCK_STREAM,0);
    if(fd == INVALID_SOCKET){
        printf("echec de demarrage du socket: %d",WSAGetLastError());
        return INVALID_SOCKET;
    }

    struct sockaddr_in sock_info;
    sock_info.sin_port = htons(port);
    sock_info.sin_family = AF_INET;
    sock_info.sin_addr.s_addr = htonl(INADDR_ANY);

  char opt = 1; 
    if(setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt))){
        printf("setsockopt a echoué");
        return INVALID_SOCKET;
    }

    if(bind(fd,(const struct sockaddr*)&sock_info,sizeof(sock_info)) == SOCKET_ERROR ){
        printf("le bind a échoué: %d",WSAGetLastError());
        return INVALID_SOCKET;
    }
    
    if(listen(fd,BACKLOG) == SOCKET_ERROR){
        printf("erreur lors de l'ecoute");
        return INVALID_SOCKET;
    }

    return fd;
}

//fonction qui accept les connection avec le client 
SOCKET accept_client(SOCKET server_fd,struct sockaddr*client_addr){
    int addr_len = sizeof(struct sockaddr_in);

    SOCKET client_fd;
    client_fd = accept(server_fd,(struct sockaddr*)client_addr,&addr_len);
    if (client_fd == INVALID_SOCKET)
    {
        printf("Erreur lors de l'acceptation:%d\n",WSAGetLastError());
        return INVALID_SOCKET;
    }

    return client_fd;
  
}

//foncion pour permet  les connections avec le serveur.
SOCKET connect_to(const char * ip,int port){

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
