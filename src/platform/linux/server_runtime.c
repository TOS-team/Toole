#include <poll.h>
#include <sys/poll.h>
#include <time.h>
#include <errno.h>
#include <unistd.h>
#include <pthread.h>

#include "discovery.h"
#include "server_runtime.h"
//cette type de fonction de fonction de sont celle de hear() et de presence dans discovery.c
// elles seront utilisé par la suite pour permettre à discovery_multiplex() de prendre les focntions hear et presence en parametre


//Hello le BOP , ici dans ce fichier ,je creér des fonctions pour gérer le logique
//concurente  de nos differentes composantes

// cette fonction prend des repères(timespec) dans le temps et renvoit la dureé ecoulé entre ses deux reperes
static int duration(struct timespec a, struct timespec b)
{
    long sec = b.tv_sec - a.tv_sec;
    long nsec = b.tv_nsec - a.tv_nsec;
    if (nsec < 0)
    {
        sec--;
        nsec += 1000000000L;
    }
    int d=(sec * 1000 + nsec / 1000000);
    return d;
}


// là c'est une focntion qui va permettre de multiplexer les fonctions presence et hear de discovery.c
int discovery_multiplex(presence_fn presence_cb, hear_fn hear_cb, context *ctx)
{
    if(!presence_cb ||!hear_cb || !ctx || !ctx->liste ||!ctx->nb) return -1;
    int sock_p= presence_socket();
    int sock_h= hear_socket();
    if (sock_p < 0 || sock_h < 0) {
        if (sock_p >= 0) close(sock_p);
        if (sock_h >= 0) close(sock_h);
        return -1;
    }

struct pollfd wait_beacon = {
    .fd = sock_h,
    .events = POLLIN //pour les données qui entrent
};
// j'enregistre le dernier envoie de beacons
struct timespec last;
clock_gettime(CLOCK_MONOTONIC,&last);

//là c'est une condition ternaire pour definir l'interval de temp entre deux beacons en ms bien sur
int interval= ctx->beacon_interval >0 ? ctx->beacon_interval:1000;

for (;;) {
    if (ctx->stop_flag && *ctx->stop_flag) {
        break;
    }

    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);

    int d=duration(last,now);
    int timeout=interval-d;
    if (timeout<0) timeout=0;

    int r=poll(&wait_beacon,1,timeout);
    if (r>0 && (wait_beacon.revents & POLLIN)) {
        hear_cb(sock_h, ctx->liste, ctx->nb);
    }
    else if (r<0 && errno != EINTR) {
        break;
    }

    clock_gettime(CLOCK_MONOTONIC, &now);
    d=duration(last,now);
    if (d >= interval)
    {
        presence_cb(sock_p, ctx->id, ctx->username, ctx->ip, ctx->port_tcp, ctx->message);
        last = now;
    }
    cleaner(ctx->liste, ctx->nb);
}
    close(sock_p);
    close(sock_h);
    return 0;
}

// Hello la BOP,cette fonction sera utilisé pour lancé une fonction sur une thread,je compte utilisé pthread 
 int run_thread()
 {
     return 0;
 }
