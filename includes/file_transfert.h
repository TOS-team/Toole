#ifndef FILE_TRANSFERT_H
#define FILE_TRANSFERT_H

#include <stddef.h>
#include <stdint.h>

int send_struct(int socket_tcp, const char *filename, uint64_t file_size);
int send_file(int socket_tcp, const char *path, const char *new_name);
int recv_struct(int socket_tcp, char *filename_out, size_t max_len, uint64_t *file_size_out);
int recv_file(int socket_tcp, const char *destination);

#endif
