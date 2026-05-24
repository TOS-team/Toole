#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

#include "file_transfert.h"
#include "network.h"

// Hello le BOP, logs detaillés pour lire le parcours du test facilement
static void step_ok(const char *step)
{
    printf("[STEP][OK] %s\n", step);
    fflush(stdout);
}

static int step_ko(const char *step, int code)
{
    fprintf(stderr, "[STEP][KO] %s (code=%d)\n", step, code);
    return code;
}

static long file_size_of(const char *path)
{
    struct stat st;
    if (stat(path, &st) != 0) return -1;
    return (long)st.st_size;
}

static int files_equal(const char *a, const char *b)
{
    FILE *fa = fopen(a, "rb");
    FILE *fb = fopen(b, "rb");
    if (!fa || !fb) {
        if (fa) fclose(fa);
        if (fb) fclose(fb);
        return 0;
    }

    int ok = 1;
    for (;;) {
        unsigned char ba[4096];
        unsigned char bb[4096];
        size_t ra = fread(ba, 1, sizeof(ba), fa);
        size_t rb = fread(bb, 1, sizeof(bb), fb);
        if (ra != rb) {
            ok = 0;
            break;
        }
        if (ra == 0) break;
        if (memcmp(ba, bb, ra) != 0) {
            ok = 0;
            break;
        }
    }

    fclose(fa);
    fclose(fb);
    return ok;
}

int main(void)
{
    printf("[INFO] file_transfer_test: debut du scenario\n");

    const uint16_t port = 44200;
    const char *src_dir = "tests/tmp_send";
    const char *dst_dir = "tests/tmp_recv";
    const char *src = "tests/tmp_send/test.txt";
    const char *dst = "tests/tmp_recv/test.txt";

    mkdir("tests", 0755);
    mkdir(src_dir, 0755);
    mkdir(dst_dir, 0755);
    unlink(src);
    unlink(dst);
    unlink("tests/tmp_recv/test_1.txt");
    unlink("tests/evil.txt");
    step_ok("preparation dossiers de test");

    FILE *f = fopen(src, "wb");
    if (!f) return step_ko("creation fichier source", 1);
    fputs("Hello le BOP, ceci est un test transfert fichier.\n", f);
    fclose(f);
    step_ok("creation fichier source");

    if (send_file(-1, src, "../evil.txt") >= 0) {
        return step_ko("rejet nom dangereux a l'envoi", 10);
    }
    if (file_size_of("tests/evil.txt") >= 0) {
        return step_ko("absence fichier traversal", 11);
    }
    step_ok("rejet nom dangereux a l'envoi");

    int server_fd = init_server_on(port);
    if (server_fd < 0) return step_ko("init serveur TCP", 2);
    step_ok("init serveur TCP");

    pid_t pid = fork();
    if (pid < 0) return step_ko("fork client", 3);
    if (pid > 0) step_ok("fork client");

    if (pid == 0) {
        sleep(1);
        int cfd = connect_to("127.0.0.1", port);
        if (cfd < 0) _exit(10);
        if (send_file(cfd, src, "test.txt") < 0) _exit(11);
        network_close(cfd);
        _exit(0);
    }

    int pfd = accept_client(server_fd);
    if (pfd < 0) return step_ko("accept client", 4);
    step_ok("accept client");

    if (recv_file(pfd, dst_dir) < 0) return step_ko("reception fichier", 5);
    step_ok("reception fichier");

    network_close(pfd);
    network_close(server_fd);
    step_ok("fermeture sockets");

    int status = 0;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) return step_ko("processus client termine proprement", 6);
    step_ok("processus client termine proprement");

    long src_size = file_size_of(src);
    long dst_size = file_size_of(dst);
    if (src_size < 0 || dst_size < 0) return step_ko("lecture tailles source/destination", 7);
    if (src_size != dst_size) return step_ko("comparaison tailles", 8);
    printf("[INFO] taille source=%ld bytes, destination=%ld bytes\n", src_size, dst_size);
    step_ok("comparaison tailles");

    if (!files_equal(src, dst)) {
        return step_ko("comparaison contenu binaire", 9);
    }
    step_ok("comparaison contenu binaire");

    printf("[OK] file transfer test reussi\n");
    return 0;
}
