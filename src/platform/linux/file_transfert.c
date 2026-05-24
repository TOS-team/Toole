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

#include "checksum.h"
#include "file_transfert.h"

#define CHUNK_SIZE 4096
#define MAX_RECEIVE_PATH 512

/*
Dans cette partie , je vais creé des helpers pour evité de repeter la logique de lecture
ou d'ecriture des fichiers ou meme  eviter d'avoir des fichiers corrompus
*/

//prototype des fonctions
int send_struct(int socket_tcp, const char *filename, uint64_t file_size);
int send_file(int socket_tcp, const char *path, const char *new_name);
int send_file_with_progress(int socket_tcp, const char *path, const char *new_name, file_transfer_progress_cb cb, void *user_data);
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
// les octets d'un entier multi-octets (ex: uint64_t) sont stockés dans l'ordre inverse
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

static int is_safe_filename(const char *filename)
{
    if (!filename || filename[0] == '\0') return 0;
    if (strcmp(filename, ".") == 0 || strcmp(filename, "..") == 0) return 0;

    for (const char *p = filename; *p; p++) {
        unsigned char c = (unsigned char)*p;
        if (c < 32 || c == 127) return 0;
        if (*p == '/' || *p == '\\') return 0;
    }

    // Hello le BOP, defense simple contre le path traversal: le recepteur
    // accepte uniquement un nom de fichier, jamais un chemin.
    if (strstr(filename, "..")) return 0;
    return 1;
}

static int ensure_directory(const char *path)
{
    if (!path || path[0] == '\0') return -1;

    struct stat st;
    if (stat(path, &st) == 0) {
        return S_ISDIR(st.st_mode) ? 0 : -1;
    }
    if (errno != ENOENT) return -1;

    return mkdir(path, 0755);
}

static int build_unique_path(char *out, size_t out_len, const char *directory, const char *filename)
{
    if (!out || !directory || !filename || out_len == 0) return -1;

    int n = snprintf(out, out_len, "%s/%s", directory, filename);
    if (n < 0 || (size_t)n >= out_len) return -1;
    if (access(out, F_OK) != 0) return 0;

    const char *dot = strrchr(filename, '.');
    size_t stem_len = dot ? (size_t)(dot - filename) : strlen(filename);
    const char *ext = dot ? dot : "";

    for (int i = 1; i < 1000; i++) {
        n = snprintf(out, out_len, "%s/%.*s_%d%s", directory, (int)stem_len, filename, i, ext);
        if (n < 0 || (size_t)n >= out_len) return -1;
        if (access(out, F_OK) != 0) return 0;
    }
    return -1;
}

/* Là maintenant , c'est la logique de transfere du fichier qui servira à envoyer et à recevoir les fichiers */
typedef struct __attribute__((packed)) {
    uint32_t name_len;
    uint64_t file_size;
} file_struct;

// Dans cette fonction, j'envoie une structure avec les infos des fichiers qui permettra de les reassemblé apres envoie
int send_struct(int socket_tcp, const char *filename, uint64_t file_size) {
    if (!filename) return -1;
    size_t name_len_size = strlen(filename);
    if (name_len_size == 0 || name_len_size > UINT32_MAX) return -1;

    file_struct one;
    one.name_len  = htonl((uint32_t)name_len_size);
    one.file_size = htonll(file_size);

    if (write_n(socket_tcp, &one, sizeof(one)) < 0) return -1;
    if (write_n(socket_tcp, filename, name_len_size) < 0) return -1;
    return 0;
}
int send_file_with_progress(int socket_tcp, const char *path, const char *new_name,
                            file_transfer_progress_cb cb, void *user_data)
{
    if (!path || !new_name) return -1;
    if (!is_safe_filename(new_name)) {
        fprintf(stderr, "[FILE] nom de fichier refuse a l'envoi: %s\n", new_name);
        return -1;
    }

    int file = open(path, O_RDONLY);
    if (file < 0) {
        perror("Erreur d'ouverture avec open");
        return -1;
    }

    struct stat st;
    if (fstat(file, &st) < 0) {
        perror("Erreur de fstat");
        close(file);
        return -1;
    }
    if (!S_ISREG(st.st_mode)) {
        fprintf(stderr, "[FILE] chemin refuse: ce n'est pas un fichier regulier\n");
        close(file);
        return -1;
    }

    uint64_t file_size = (uint64_t)st.st_size;

    if (send_struct(socket_tcp, new_name, file_size) < 0) {
        close(file);
        return -1;
    }

    uint32_t crc = crc32_init();
    uint64_t sent_total = 0;
    if (cb) cb(0, file_size, user_data);

    uint8_t buf[CHUNK_SIZE];
    for (;;) {
        ssize_t r = read(file, buf, sizeof(buf));
        if (r < 0) {
            perror("read");
            close(file);
            return -1;
        }
        if (r == 0) break;

        crc = crc32_update(crc, buf, (size_t)r);

        if (write_n(socket_tcp, buf, (size_t)r) < 0) {
            close(file);
            return -1;
        }
        sent_total += (uint64_t)r;
        if (cb) cb(sent_total, file_size, user_data);
    }
    close(file);

    uint32_t final_crc = htonl(crc32_finalize(crc));
    if (write_n(socket_tcp, &final_crc, sizeof(final_crc)) < 0) return -1;

    if (cb) cb(file_size, file_size, user_data);
    return 0;
}

int send_file(int socket_tcp, const char *path, const char *new_name)
{
    return send_file_with_progress(socket_tcp, path, new_name, NULL, NULL);
}

int recv_struct(int socket_tcp, char *filename_out, size_t max_len, uint64_t *file_size_out)
{
    if (!filename_out || !file_size_out || max_len == 0) return -1;

    file_struct one;
    if (read_n(socket_tcp, &one, sizeof(one)) < 0) return -1;

    uint32_t name_len = ntohl(one.name_len);
    *file_size_out = ntohll(one.file_size);

    if (name_len == 0 || name_len >= max_len) return -1;
    if (read_n(socket_tcp, filename_out, name_len) < 0) return -1;

    filename_out[name_len] = '\0';
    if (!is_safe_filename(filename_out)) {
        fprintf(stderr, "[FILE] nom de fichier recu refuse: %s\n", filename_out);
        return -1;
    }
    return 0;
}

int recv_file(int socket_tcp, const char *destination)
{
    if (ensure_directory(destination) < 0) {
        fprintf(stderr, "[FILE] destination invalide: %s\n", destination ? destination : "(null)");
        return -1;
    }

    char filename[256];
    uint64_t file_size;
    if (recv_struct(socket_tcp, filename, sizeof(filename), &file_size) < 0) return -1;

    char path[MAX_RECEIVE_PATH];
    if (build_unique_path(path, sizeof(path), destination, filename) < 0) return -1;

    int file = open(path, O_WRONLY | O_CREAT | O_EXCL, 0644);
    if (file < 0) return -1;

    uint32_t crc = crc32_init();

    uint8_t buffer[CHUNK_SIZE];
    uint64_t total = 0;
    while (total < file_size) {
        size_t to_read = (file_size - total > CHUNK_SIZE) ? CHUNK_SIZE : (size_t)(file_size - total);
        if (read_n(socket_tcp, buffer, to_read) < 0) {
            close(file);
            unlink(path);
            return -1;
        }

        crc = crc32_update(crc, buffer, to_read);

        size_t written = 0;
        while (written < to_read) {
            ssize_t w = write(file, buffer + written, to_read - written);
            if (w < 0) {
                if (errno == EINTR) continue;
                close(file);
                unlink(path);
                return -1;
            }
            written += (size_t)w;
        }
        total += to_read;
    }
    close(file);

    uint32_t expected_crc_net;
    if (read_n(socket_tcp, &expected_crc_net, sizeof(expected_crc_net)) < 0) {
        fprintf(stderr, "[FILE] impossible de lire le CRC32 du fichier %s\n", filename);
        unlink(path);
        return -1;
    }
    uint32_t expected_crc = ntohl(expected_crc_net);
    uint32_t computed_crc = crc32_finalize(crc);

    if (computed_crc != expected_crc) {
        fprintf(stderr, "[FILE] CRC32 mismatch pour %s: attendu=0x%08X recu=0x%08X => fichier corrompu\n",
                filename, expected_crc, computed_crc);
        unlink(path);
        return -2;
    }

    fprintf(stderr, "[FILE] CRC32 OK pour %s (0x%08X) -> %s\n", filename, computed_crc, path);
    return 0;
}
