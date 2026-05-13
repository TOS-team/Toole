#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <time.h>

#pragma comment(lib,"Ws2_32.lib")//pour indiquer au compilateur d'ajouter la bib ws2_32.lib lors de la compilation
#define PORT 47272
#define BROADCAST_IP "255.255.255.255"//address de diffusion



// En gros ,cette fonction ci-dessous permet sa presence aux autres pc qui ecoutent sur le port.
int presence(char *ip,int port_tcp,char *message,char *username,int id);
//------------------------------------------------------------------------------------
// Ici cette le prototype de la func qui permet d'ecouter sur le resau

//generer des id pour les pc.
char * id_generertor();

typedef struct {
    char id[37];
    char username[64];
    char ip[16];
    int  port_tcp;
    char message[128];
    time_t last_time;
} devices;

/*Implementation des fonction */

void cleaner(devices *liste,int *nb){
    time_t now = time(NULL);
    int keepCount = 0;

    for(int i = 0; i < *nb; i++){
        if ((now - liste[i].last_time) <= 10){
            liste[keepCount] = liste[i];
            keepCount++;
        }

    }
    *nb = keepCount;
}
//fonction pour ecouter sur le port .

int head(devices *liste,int *nb){
    char buffer[256];

    SOCKET head_sock = socket(AF_INET,SOCK_DGRAM,0);
    if(head_sock == INVALID_SOCKET){
        printf("creation de la socket  a échoué : %d", WSAGetLastError());
        WSACleanup();
        exit(1);
    }

    //  Configuration de l'adresse de réception
    struct sockaddr_in recvAddr;
    recvAddr.sin_family = AF_INET;
    recvAddr.sin_port = htons(PORT);
    recvAddr.sin_addr.s_addr = INADDR_ANY;

    // Liaison (Bind) AVEC le socket (head_sock) creer plus haut
    if (bind(head_sock, (struct sockaddr*)&recvAddr, sizeof(recvAddr)) == SOCKET_ERROR) {
        printf("Erreur bind: %d\n", WSAGetLastError());
        closesocket(head_sock);
        WSACleanup();
        return 1;
    }

    //   réception
        int bytesReceived = recvfrom(head_sock, buffer, sizeof(buffer) - 1, 0,
                                     (struct sockaddr*)&recvAddr, &recvAddr);

        if (bytesReceived > 0) {
            char *ip_client = inet_ntoa(recvAddr.sin_addr);
            if(strcmp(ip_client,"127.0.0.1")==0){
                printf("c'est mon propre message");
                return;
            }else{
            buffer[bytesReceived] = '\0';

               if (strncmp(buffer, "toole", 5) == 0) {
                devices d;
                // Je  parse les beacons recues pour le mettre  dans la structure device que j'ai creé
                sscanf(buffer, "toole|%36[^|]|%63[^|]|%15[^|]|%d|%127[^\n]", d.id, d.username, d.ip, &d.port_tcp, d.message);
                d.last_time=time(NULL);
                /*là pour eviter les doublons de beacons, je verifie la liste, si l'id d'un nouveau becons est deja present dans la liste ,
                 je le suprime et dans le cas contraire , je l'ajoute imediatement
                    */
                int index = -1;
                for (int i = 0; i < nb; i++) {
                    if (strcmp(liste[i].id, d.id) == 0) {
                        index = i;
                        break;
                    }
                }
                if (index != -1) {
                    liste[index] = d;
                }
                else if (nb < 100) {
                    liste[*nb] = d;
                    nb++;
                }
            }
        }


        } else if (bytesReceived == SOCKET_ERROR) {
            printf("Erreur recvfrom: %d\n", WSAGetLastError());

        }

}

int presence(char *ip,int port_tcp,char *message,char *username,int id){

    char beacan[256];
    snprintf(beacan,sizeof(beacan), "toole|%s|%s|%s|%d|%s",id,username,ip,port_tcp,message);

    SOCKET sock_envoi = socket(AF_INET,SOCK_DGRAM,0);
    if(sock_envoi == INVALID_SOCKET){
        printf("creation de la socket  a échoué : %d", WSAGetLastError());
        WSACleanup();
        exit(1);
    }
    //bro windows a par defaut bloque le broadcast donc cette petite manipe nous permet de debloque le broadcast sur windows
    BOOL broadcastPermission = TRUE;
    if(setsockopt(sock_envoi,SOL_SOCKET,SO_BROADCAST,(char*)&broadcastPermission,sizeof(broadcastPermission)) == SOCKET_ERROR){
        printf("permition de broadcast  a échoué : %d", WSAGetLastError());
        closesocket(sock_envoi);
        WSACleanup();
        exit(1);
    }
    u_long mode = 1;
    if(ioctlsocket(sock_envoi,FIONBIO,&mode) == SOCKET_ERROR){
        printf("erreur d'ecoute: %d",WSAGetLastError());
        exit(1);
    }

    struct sockaddr_in infosock_sa;
    infosock_sa.sin_family = AF_INET;
    infosock_sa.sin_port = htons(PORT);
    infosock_sa.sin_addr.s_addr = INADDR_BROADCAST;


    if (sendto(sock_envoi, beacan, strlen(beacan), 0, (struct sockaddr *)&infosock_sa, sizeof(infosock_sa)) == SOCKET_ERROR) {
            printf("sendto() a échoué : %d", WSAGetLastError());

    }


    // comme tu le sais : si tu a ouvert forcer tu vas  Fermeture.ainsi cette fonction permet de liberer les resource.
    closesocket(sock_envoi);
    WSACleanup();
    return 0;
    }

char * id_generertor(){
    int nombre;
    char chaine[10];
    char computerName[MAX_COMPUTERNAME_LENGTH + 1];
    DWORD size = sizeof(computerName);
    char *namepc = malloc(MAX_COMPUTERNAME_LENGTH * sizeof(char));
    if(namepc == NULL){
        fprintf(stderr,"erreur de l'allocstion de namepc");
        return "INCONNU";
    }

    //Avec cette fonction{GetComputerName} je recupere le nom de ta machine et avec {strcpy} je copie le nom de tas machine dans la variable computerName
    if (GetComputerName(computerName,&size))
    {
        strcpy(namepc,computerName);

    }else {
        printf("Erreur lors de la recuperation du nom (code : %lu)\n", GetLastError());
        return "INCONNU";
    }
    srand(time(NULL));
    nombre = rand() % 1001;//generation d'un nombre alleatoire pour faire l'id
    sprintf(chaine, "%d", nombre);
        //  Copier computerName dans namepc
    strcpy(namepc, computerName);

    //  Concaténer chaine à la suite de namepc
    strcat(namepc, chaine);

    return namepc;//je retourne l'id avec ce format {NOMPCNOMBRE} EX:DELL57665555 NB:5555 est le nombre generer et DELL5766 nom de la machine.
}
