/*
 * PS4 Studio companion — console entry point.
 *
 *  *** UNTESTED SKELETON ***  This file has never been compiled against the OpenOrbis toolchain or
 *  run on a console (neither was available when it was written). The portable protocol logic it
 *  relies on (src/proto.c) IS tested on the host (make host-test). Treat everything below as a
 *  starting point that must be built and verified on real, compatible hardware by someone who
 *  already runs a homebrew environment. It must not be presented as working until that happens.
 *
 * Design (see README.md):
 *  - runs as a normal homebrew app; no privileged services, no shell, no arbitrary file access;
 *  - shows a one-time 8-digit pairing code; 3 wrong attempts lock pairing until restart;
 *  - allowlisted commands only (ps4s_command_allowed); themes stored under ONE app data dir, ids
 *    validated with ps4s_valid_theme_id so a theme id can never address another path;
 *  - rendering and controller navigation are NOT implemented here.
 */
#include "proto.h"
#include <stdio.h>
#include <string.h>

#define LISTEN_PORT 9777
#define APP_DATA_DIR "/data/ps4studio" /* assumption: unverified on-console location */

/* TODO(OpenOrbis): replace with real kernel/net initialisation, e.g. sceKernelLoadStartModule for libSceNet,
 * sceNetInit, sceNetSocket/Bind/Listen/Accept. Names/headers must be taken from the toolchain's samples. */
static int net_listen(int port) { (void)port; return -1; }
static int net_accept(int srv) { (void)srv; return -1; }
static int net_read_frame(int c, char *buf, size_t cap, size_t *len) { (void)c; (void)buf; (void)cap; (void)len; return -1; }
static int net_write_frame(int c, const char *buf, size_t len) { (void)c; (void)buf; (void)len; return -1; }

/* Store a theme: validates id, bounds size, writes only to APP_DATA_DIR/themes/<id>.json. */
static int store_theme(const char *id, const char *json, size_t len) {
  if (!ps4s_valid_theme_id(id) || len == 0 || len > 256 * 1024) return -1;
  char path[160];
  snprintf(path, sizeof path, "%s/themes/%s.json", APP_DATA_DIR, id);
  FILE *f = fopen(path, "wb");
  if (!f) return -1;
  size_t w = fwrite(json, 1, len, f);
  fclose(f);
  return w == len ? 0 : -1;
  /* TODO: also validate the JSON structure (schema in packages/theme-engine/theme.schema.json). */
}

int main(void) {
  /* TODO: generate the pairing code from a cryptographically secure source (sceKernelGetRandom / equivalent)
   * and draw it on screen. A fixed value is used only so this skeleton is self-consistent. */
  const char *code = "00000000";
  if (!ps4s_valid_code(code)) return 1;

  int srv = net_listen(LISTEN_PORT);
  if (srv < 0) { printf("[ps4studio] network not available in this skeleton\n"); return 1; }
  unsigned attempts = 0;
  for (;;) {
    int c = net_accept(srv);
    if (c < 0) break;
    /* 1. read hello  2. send hello_ack {protocol, version, nonce, features}
     * 3. read pair; compare with ps4s_client_proof + ps4s_ct_equal; count attempts (max 3)
     * 4. send pair_ack with ps4s_server_proof; derive key with ps4s_session_key
     * 5. loop: read frame, ps4s_frame_verify(key,'c',...), require seq == last+1,
     *          require ps4s_command_allowed(cmd), dispatch (status/list/put/delete/config/diagnostics),
     *          reply with ps4s_frame_mac(key,'s',...). Any failure: close the connection. */
    (void)attempts; (void)store_theme; (void)net_read_frame; (void)net_write_frame;
  }
  return 0;
}
