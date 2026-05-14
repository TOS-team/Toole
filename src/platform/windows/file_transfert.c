#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <mswsock.h>


#define SERVER_PORT 42422
#define BACKLOG 16


// cette fonction garentie que les donner sont bien envoier

int write_n(SOCKET fd,char *buffer,size_t n){
    size_t total_sent = 0;
    while (total_sent < n)
    {
        int bytes = send(fd,buffer + total_sent,(int)(n-total_sent),0);
        if(bytes == SOCKET_ERROR) return -1;
        total_sent +=bytes;
    }

    return 0;
}

//cette s'assure que les donnees sont bien recu.
int read_n(SOCKET fd,char *buffer,size_t n){
    size_t total_recv = 0;
    while (total_recv < n)
    {
        int bytes = recv(fd,buffer + total_recv,(int)(n-total_recv),0);
        if(bytes == SOCKET_ERROR) return -1;
        if(bytes == 0) return -2;//connection coupée
        total_recv +=bytes;
    }

    return 0;
}

int network_send_struct(SOCKET fd,const void * data,size_t size){
    if(write_n(fd,(const char *)data,size) != 0)
    {
        printf("Erreur lors de l'envoi de la structure\n");
        return -1;
    }

    return 0;
}

int network_recv_struct(SOCKET fd,const void * data,size_t size){
    int res = read_n(fd,(char *)data,size);
    if(res == -1){
        printf("Erreur lors de la réception\n");
        return -1;
    }
    if(res == -2){
        printf("ls connection est fermet");
    }

    return 0;
}

//fonction envoi des fichies
int network_send_file(SOCKET fd,const char * path){
    HANDLE hfile = CreateFile(path,GENERIC_READ,FILE_SHARE_READ,NULL,OPEN_EXISTING,FILE_FLAG_SEQUENTIAL_SCAN,NULL);
    if(hfile == INVALID_HANDLE_VALUE) return -1;

    //recuperation de la taille du fchiers
    LARGE_INTEGER filesize ;
    GetFileSize(hfile,&filesize);//fonction permetant de recuperer

    //envoie de la taille du fichiers
    if(write_n(fd,&filesize.QuadPart,sizeof(filesize.QuadPart)) < 0){
        CloseHandle(hfile);
        return -1;
    }

    //boucle por gerer les fichiers > 2 go.
    OVERLAPPED ol = {0};
    unsigned __int64 reste = filesize.QuadPart;
    unsigned __int64 total =0;
    const DWORD MAX_CHUNK = 2147483646 ; //limites de windows sa veux dire que c'est la limites dedonne que windows peut envoier en meme temps .


    while (reste > 0)
    {
        DWORD bytes_a_envoier = (reste > MAX_CHUNK) ? MAX_CHUNK : (DWORD)reste;
        ol.Offset = (DWORD)(total & 0xFFFFFFFF );
        ol.OffsetHigh = (DWORD)(total >> 32);

        if(!TransmitFile(fd,hfile,bytes_a_envoier,0,&ol,NULL,0)){
            if(WSAGetLastError() != 0) break;

            DWORD bytes;
            GetOverlappedResult((HANDLE)fd,&ol,&bytes,TRUE);
        }


        total += bytes_a_envoier;
        reste -= bytes_a_envoier;

    }


    CloseHandle(hfile);
    return (reste == 0) ? 0 : -1;
}

//reception des fichiers.
int network_recv_file(SOCKET fd, const char *dst_path) {
    long long fileSize = 0;

    //  Lire la taille d'abord
    // On reçoit le nombre d'octets que l'envoyeur a calculé avec GetFileSizeEx.
    if (read_n(fd, &fileSize, sizeof(fileSize)) < 0) return -1;

    //  Ouvrir le fichier local en mode écriture binaire ("wb")
    FILE* f = fopen(dst_path, "wb");
    if (!f) return -1;

    char buffer[65536]; // Tampon de 64 Ko pour stocker temporairement les données reçues.
    long long totalRecu = 0;

    //  Boucle de réception
    while (totalRecu < fileSize) {
        // On calcule combien d'octets lire pour ne pas dépasser la taille du buffer ni la fin du fichier.
        int aLire = (fileSize - totalRecu > sizeof(buffer)) ? sizeof(buffer) : (int)(fileSize - totalRecu);

        // recv() récupère les données arrivant de la socket.
        int n = recv(fd, buffer, aLire, 0);

        // Si n <= 0, soit la connexion est coupée (0), soit il y a une erreur (-1).
        if (n <= 0) break;

        // On écrit les octets reçus dans le fichier local.
        fwrite(buffer, 1, n, f);

        totalRecu += n; // On met à jour le compteur global.
    }

    fclose(f); // Ferme le fichier une fois terminé.
    // Si on a reçu autant d'octets que prévu, c'est un succès (0).
    return (totalRecu == fileSize) ? 0 : -1;
}
