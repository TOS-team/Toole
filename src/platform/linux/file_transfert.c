<<<<<<< HEAD
#include <stddef.h>
#include <stdint.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <arpa/inet.h>
#include <string.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <errno.h>

#define CHUNK_SIZE 4096

/*
Dans cette partie , je vais creé des helpers pour evité de repeter la logique de lecture
ou d'ecriture des fichiers ou meme  eviter d'avoir des fichiers corrompus
*/

//prototype des fonctions
int send_struct(int socket_tcp, const char *filename, uint64_t file_size);
int send_file(int socket_tcp, const char *path, const char *new_name);
int recv_struct(int socket_tcp, char *filename_out, ssize_t max_len, uint64_t *file_size_out);
int recv_file(int socket_tcp, const char *destination);



static int write_n(int socket_tcp,const void *buffer,size_t n){
    const uint8_t *p=(const uint8_t *)buffer;
    size_t sent=0;
    while (sent<n) {
        ssize_t w=send(socket_tcp,p+sent,n-sent,0);
        if(w<0){
            if (errno == EINTR) continue;
            return -1;
        }
        if (w==0) return -1;
        sent+=(size_t)w;
    }
    return 0;
}

static int read_n(int socket_tcp,void *buffer,size_t n){
    uint8_t *p=(uint8_t *)buffer;
    size_t recvv=0;
    while (recvv<n) {
        ssize_t r=recv(socket_tcp,p+recvv,n-recvv,0);
        if(r<0){
            if (errno == EINTR) continue;
            return -1;
        }
        if (r==0) return -1;
        recvv+=(size_t)r;
    }
    return 0;
}

//on convertit l'ordre des octets avant de les envoyés pour optimiser le compatibilité entre architecture little-endian et big-endian
// les octets d’un entier multi-octets (ex: uint64_t) sont stockés dans l’ordre inverse
// sans ca le recepteur pourrait reconstruire une valeur incorrecte à la reception
static uint64_t htonll(uint64_t x) {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return ((uint64_t)htonl((uint32_t)(x >> 32))) | ((uint64_t)htonl((uint32_t)(x & 0xFFFFFFFFULL)));
#else
    return x;
#endif
}
//Voici l'equivalent de htonll pour reconstruire l'information à la reception
static uint64_t ntohll(uint64_t x) {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return ((uint64_t)ntohl((uint32_t)(x & 0xFFFFFFFFULL)) << 32) |
            ntohl((uint32_t)(x >> 32));
#else
    return x;
#endif
}


/* Là maintenant , c'est la logique de transfere du fichier qui servira à envoyer et à recevoir les fichiers */
typedef struct __attribute__((packed)){
    uint32_t name_len;
    uint64_t file_size;
    } file_struct;

// Dans cette fonction, j'envoie une structure avec les infos des fichiers qui permettra de les reassemblé apres envoie
int send_struct(int socket_tcp,const char *filename,uint64_t file_size){
    if(!filename)return -1;
    size_t name_len_size=strlen(filename);
    if(name_len_size==0 ||name_len_size> UINT32_MAX) return -1;

    file_struct one;
    one.name_len  = htonl((uint32_t)name_len_size);
    one.file_size = htonll(file_size);

    //let's go envoyons l'en tete
    if(write_n(socket_tcp,&one,sizeof(one))<0) return -1;
    if (write_n(socket_tcp, filename, name_len_size) < 0) return -1;
    return 0;
}
//Hello la BOP, encore Gérard sur ce nouveau fichier , je cree cette focntion pour l'envoie de fichier à transfere un socket TCP dejà existant
int send_file(int socket_tcp,const char *path,const char *new_name){
    if (!path || !new_name) return -1;//verification des parametres
    int file = open(path, O_RDONLY); // ouverture du fcihier en lecture seule
        if (file < 0) {
            perror("Erreur d'ouverture avec open");
            return -1;
        }
        // cette structure contiendra les metadonnées de chaque fichier
        struct stat st;
        if (fstat(file, &st) < 0) {
            perror("Erreur de fstat");
            close(file);
            return -1;
        }

        uint64_t file_size = (uint64_t)st.st_size;

        if (send_struct(socket_tcp, new_name, file_size) < 0) {
                close(file);
                return -1;
            }
        uint8_t buf[CHUNK_SIZE];
        // eh  ben là , for(;;) permet de lancer une boucle infini,équivalent à while(1)
        for (;;) {
                ssize_t r = read(file, buf, sizeof(buf));
                if (r < 0) {
                    perror("read");
                    close(file);
                    return -1;
                }
                if (r == 0) break;
                if (write_n(socket_tcp, buf, (size_t)r) < 0) {
                    close(file);
                    return -1;
                }
        }
        close(file);
        return 0;
}

// là cette focntion est l'equivalent de send_struct à la reception
int recv_struct(int socket_tcp,char *filename_out,ssize_t max_len,uint64_t *file_size_out){
    //
    file_struct one;
    if (read_n(socket_tcp,&one, sizeof(one))<0) return -1;

    uint32_t name_len = ntohl(one.name_len);
    *file_size_out = ntohll(one.file_size);

    if(name_len == 0 || name_len >= max_len) return -1;
    if(read_n(socket_tcp, filename_out, name_len) < 0) return -1;

    filename_out[name_len] = '\0';
    return 0;
}

// ici, c'est la focntion qui permettra de recevoir le fichier envoyé
int recv_file(int socket_tcp,const char *destination){
    char filename[256];
    uint64_t file_size;
    if(recv_struct(socket_tcp, filename, sizeof(filename), &file_size) < 0) return -1;

    char path[512];
    snprintf(path, sizeof(path), "%s/%s", destination, filename);

    int file = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if(file < 0) return -1;
    uint8_t buffer[CHUNK_SIZE];
    uint64_t total = 0;
    while(total < file_size) {
        size_t to_read = (file_size - total > CHUNK_SIZE) ? CHUNK_SIZE : (size_t)(file_size - total);
        if(read_n(socket_tcp, buffer, to_read) < 0) {
            close(file);
            return -1;
        }
        if(write(file, buffer, to_read) < 0) {
            close(file);
            return -1;
        }
        total+=to_read;
    }
    close(file);
    return 0;
}
=======
#include <stddef.h>
#include <stdint.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <arpa/inet.h>
#include <string.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <errno.h>

#define CHUNK_SIZE 4096

/*
Dans cette partie , je vais creé des helpers pour evité de repeter la logique de lecture
ou d'ecriture des fichiers ou meme  eviter d'avoir des fichiers corrompus
*/

//prototype des fonctions
int send_struct(int socket_tcp, const char *filename, uint64_t file_size);
int send_file(int socket_tcp, const char *path, const char *new_name);
int recv_struct(int socket_tcp, char *filename_out, size_t max_len, uint64_t *file_size_out);
int recv_file(int socket_tcp, const char *destination);



static int write_n(int socket_tcp,const void *buffer,size_t n){
    const uint8_t *p=(const uint8_t *)buffer;
    size_t sent=0;
    while (sent<n) {
        ssize_t w=send(socket_tcp,p+sent,n-sent,0);
        if(w<0){
            if (errno == EINTR) continue;
            return -1;
        }
        if (w==0) return -1;
        sent+=(size_t)w;
    }
    return 0;
}

static int read_n(int socket_tcp,void *buffer,size_t n){
    uint8_t *p=(uint8_t *)buffer;
    size_t recvv=0;
    while (recvv<n) {
        ssize_t r=recv(socket_tcp,p+recvv,n-recvv,0);
        if(r<0){
            if (errno == EINTR) continue;
            return -1;
        }
        if (r==0) return -1;
        recvv+=(size_t)r;
    }
    return 0;
}

//on convertit l'ordre des octets avant de les envoyés pour optimiser le compatibilité entre architecture little-endian et big-endian
// les octets d’un entier multi-octets (ex: uint64_t) sont stockés dans l’ordre inverse
// sans ca le recepteur pourrait reconstruire une valeur incorrecte à la reception
static uint64_t htonll(uint64_t x) {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
return ((uint64_t)htonl((uint32_t)(x & 0xFFFFFFFFULL)) << 32) |
       ((uint64_t)htonl((uint32_t)(x >> 32)));
#else
    return x;
#endif
}
//Voici l'equivalent de htonll pour reconstruire l'information à la reception
static uint64_t ntohll(uint64_t x) {
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
return ((uint64_t)ntohl((uint32_t)(x & 0xFFFFFFFFULL)) << 32) |
        (uint64_t)ntohl((uint32_t)(x >> 32));

#else
    return x;
#endif
}


/* Là maintenant , c'est la logique de transfere du fichier qui servira à envoyer et à recevoir les fichiers */
typedef struct __attribute__((packed)){
    uint32_t name_len;
    uint64_t file_size;
    } file_struct;

// Dans cette fonction, j'envoie une structure avec les infos des fichiers qui permettra de les reassemblé apres envoie
int send_struct(int socket_tcp,const char *filename,uint64_t file_size){
    if(!filename)return -1;
    size_t name_len_size=strlen(filename);
    if(name_len_size==0 ||name_len_size> UINT32_MAX) return -1;

    file_struct one;
    one.name_len  = htonl((uint32_t)name_len_size);
    one.file_size = htonll(file_size);

    //let's go envoyons l'en tete
    if(write_n(socket_tcp,&one,sizeof(one))<0) return -1;
    if (write_n(socket_tcp, filename, name_len_size) < 0) return -1;
    return 0;
}
//Hello la BOP, encore Gérard sur ce nouveau fichier , je cree cette focntion pour l'envoie de fichier à transfere un socket TCP dejà existant
int send_file(int socket_tcp,const char *path,const char *new_name){
    if (!path || !new_name) return -1;//verification des parametres
    int file = open(path, O_RDONLY); // ouverture du fcihier en lecture seule
        if (file < 0) {
            perror("Erreur d'ouverture avec open");
            return -1;
        }
        // cette structure contiendra les metadonnées de chaque fichier
        struct stat st;
        if (fstat(file, &st) < 0) {
            perror("Erreur de fstat");
            close(file);
            return -1;
        }

        uint64_t file_size = (uint64_t)st.st_size;

        if (send_struct(socket_tcp, new_name, file_size) < 0) {
                close(file);
                return -1;
            }
        uint8_t buf[CHUNK_SIZE];
        // eh  ben là , for(;;) permet de lancer une boucle infini,équivalent à while(1)
        for (;;) {
                ssize_t r = read(file, buf, sizeof(buf));
                if (r < 0) {
                    perror("read");
                    close(file);
                    return -1;
                }
                if (r == 0) break;
                if (write_n(socket_tcp, buf, (size_t)r) < 0) {
                    close(file);
                    return -1;
                }
        }
        close(file);
        return 0;
}

// là cette focntion est l'equivalent de send_struct à la reception
int recv_struct(int socket_tcp,char *filename_out,size_t max_len,uint64_t *file_size_out){
    //
    file_struct one;
    if (read_n(socket_tcp,&one, sizeof(one))<0) return -1;

    uint32_t name_len = ntohl(one.name_len);
    *file_size_out = ntohll(one.file_size);

    if(name_len == 0 || name_len >= max_len) return -1;
    if(read_n(socket_tcp, filename_out, name_len) < 0) return -1;

    filename_out[name_len] = '\0';
    return 0;
}

// ici, c'est la focntion qui permettra de recevoir le fichier envoyé
int recv_file(int socket_tcp,const char *destination){
    char filename[256];
    uint64_t file_size;
    if(recv_struct(socket_tcp, filename, sizeof(filename), &file_size) < 0) return -1;

    char path[512];
    int n = snprintf(path, sizeof(path), "%s/%s", destination, filename);
    if (n < 0 || (size_t)n >= sizeof(path)) return -1;

    int file = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if(file < 0) return -1;
    uint8_t buffer[CHUNK_SIZE];
    uint64_t total = 0;
    while(total < file_size) {
        size_t to_read = (file_size - total > CHUNK_SIZE) ? CHUNK_SIZE : (size_t)(file_size - total);
        if(read_n(socket_tcp, buffer, to_read) < 0) {
            close(file);
            return -1;
        }
        size_t written = 0;
        while (written < to_read) {
            ssize_t w = write(file, buffer + written, to_read - written);
            if (w < 0) {
                if (errno == EINTR) continue;
                close(file);
                return -1;
            }
            written += (size_t)w;
        }
        total+=to_read;
    }
    close(file);
    return 0;
}
>>>>>>> 4f2ec6ce32eb8f8092ad46be7fd76220b4f353dd
