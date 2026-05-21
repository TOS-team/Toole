#ifndef CHECKSUM_H
#define CHECKSUM_H

#include <stdint.h>
#include <stddef.h>

// Hello le BOP, ici on pose un CRC32 standard (IEEE 802.3) pour verifier
// l'integrite des fichiers transferes sur le reseau local
// le polynome 0xEDB88320 est la forme reflechie du polynome CRC-32 classique
// ca permet de calculer le checksum chunk par chunk sans tout garder en memoire

// là on genere la table de 256 entrees au premier appel
// chaque entree represente le CRC partiel d'un octet donné
static uint32_t crc32_table_data[256];
static int crc32_table_built = 0;

static inline void crc32_build_table(void)
{
    if (crc32_table_built) return;
    for (uint32_t i = 0; i < 256; i++) {
        uint32_t crc = i;
        for (int j = 0; j < 8; j++) {
            // si le bit de poids faible est 1, on XOR avec le polynome
            // sinon on decale simplement vers la droite
            crc = (crc & 1) ? ((crc >> 1) ^ 0xEDB88320UL) : (crc >> 1);
        }
        crc32_table_data[i] = crc;
    }
    crc32_table_built = 1;
}

// on initialise le CRC à 0xFFFFFFFF (standard CRC32)
static inline uint32_t crc32_init(void)
{
    crc32_build_table();
    return 0xFFFFFFFFUL;
}

// là on met à jour le CRC avec un nouveau bloc de donnees
// on peut appeler cette fonction autant de fois qu'on veut avec des chunks successifs
static inline uint32_t crc32_update(uint32_t crc, const void *data, size_t len)
{
    const uint8_t *p = (const uint8_t *)data;
    for (size_t i = 0; i < len; i++) {
        crc = crc32_table_data[(crc ^ p[i]) & 0xFF] ^ (crc >> 8);
    }
    return crc;
}

// on finalise le CRC en inversant tous les bits (standard CRC32)
static inline uint32_t crc32_finalize(uint32_t crc)
{
    return crc ^ 0xFFFFFFFFUL;
}

#endif
