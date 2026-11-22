/* PS4 Studio companion: portable protocol core (protocol v1). No OS or console APIs.
 * Mirrors crates/core/src/companion_proto.rs; cross-checked by tests/vectors.json. */
#ifndef PS4S_PROTO_H
#define PS4S_PROTO_H
#include <stddef.h>
#include <stdint.h>

#define PS4S_PROTOCOL 1
#define PS4S_MAX_FRAME (400 * 1024)

void ps4s_sha256(const uint8_t *data, size_t len, uint8_t out[32]);
/* HMAC-SHA256 over the concatenation of n parts. */
void ps4s_hmac(const uint8_t *key, size_t keylen, const uint8_t *const *parts, const size_t *lens, size_t n, uint8_t out[32]);

void ps4s_to_hex(const uint8_t *b, size_t n, char *out /* 2n+1 bytes */);
int ps4s_ct_equal(const char *a, const char *b); /* constant-time for equal-length strings; 0 if lengths differ */

int ps4s_valid_code(const char *code);                 /* exactly 8 ASCII digits */
int ps4s_valid_theme_id(const char *id);               /* ^[a-z0-9][a-z0-9-]{0,39}$ */
int ps4s_command_allowed(const char *cmd);             /* the 7 allowlisted commands only */
int ps4s_config_key_allowed(const char *key);          /* active_theme, show_clock */

/* Pairing (hex nonces exactly as exchanged). Output buffers: proofs 65 bytes, key 32 bytes. */
void ps4s_client_proof(const char *code, const char *cn, const char *sn, char out[65]);
void ps4s_server_proof(const char *code, const char *cn, const char *sn, char out[65]);
void ps4s_session_key(const char *code, const char *cn, const char *sn, uint8_t out[32]);

/* dir: 'c' = client->server ("c2s|"), 's' = server->client ("s2c|"). out: 65 bytes. */
void ps4s_frame_mac(const uint8_t key[32], char dir, const char *body, size_t bodylen, char out[65]);
int ps4s_frame_verify(const uint8_t key[32], char dir, const char *body, size_t bodylen, const char *mac_hex);

#endif
