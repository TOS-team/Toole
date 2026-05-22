#ifndef FILE_TRANSFERT_H
#define FILE_TRANSFERT_H

#include <stddef.h>
#include <stdint.h>

typedef void (*file_transfer_progress_cb)(uint64_t sent, uint64_t total, void *user_data);

int send_struct(int socket_tcp, const char *filename, uint64_t file_size);
int send_file(int socket_tcp, const char *path, const char *new_name);
int send_file_with_progress(int socket_tcp, const char *path, const char *new_name, file_transfer_progress_cb cb, void *user_data);
int recv_struct(int socket_tcp, char *filename_out, size_t max_len, uint64_t *file_size_out);
int recv_file(int socket_tcp, const char *destination);

#endif
