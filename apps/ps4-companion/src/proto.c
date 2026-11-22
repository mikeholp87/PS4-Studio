#include "proto.h"
#include <string.h>

/* ---- SHA-256 (FIPS 180-4), compact implementation ---- */
static const uint32_t K[64] = {
 0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
 0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
 0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
 0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
 0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
 0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
 0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
 0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2};
#define ROR(x, n) (((x) >> (n)) | ((x) << (32 - (n))))

typedef struct { uint32_t h[8]; uint8_t buf[64]; size_t blen; uint64_t total; } sha_ctx;

static void sha_block(sha_ctx *c, const uint8_t *p) {
  uint32_t w[64], a, b, cc, d, e, f, g, h, t1, t2;
  for (int i = 0; i < 16; i++) w[i] = ((uint32_t)p[4*i] << 24) | ((uint32_t)p[4*i+1] << 16) | ((uint32_t)p[4*i+2] << 8) | p[4*i+3];
  for (int i = 16; i < 64; i++) {
    uint32_t s0 = ROR(w[i-15], 7) ^ ROR(w[i-15], 18) ^ (w[i-15] >> 3);
    uint32_t s1 = ROR(w[i-2], 17) ^ ROR(w[i-2], 19) ^ (w[i-2] >> 10);
    w[i] = w[i-16] + s0 + w[i-7] + s1;
  }
  a = c->h[0]; b = c->h[1]; cc = c->h[2]; d = c->h[3]; e = c->h[4]; f = c->h[5]; g = c->h[6]; h = c->h[7];
  for (int i = 0; i < 64; i++) {
    t1 = h + (ROR(e, 6) ^ ROR(e, 11) ^ ROR(e, 25)) + ((e & f) ^ (~e & g)) + K[i] + w[i];
    t2 = (ROR(a, 2) ^ ROR(a, 13) ^ ROR(a, 22)) + ((a & b) ^ (a & cc) ^ (b & cc));
    h = g; g = f; f = e; e = d + t1; d = cc; cc = b; b = a; a = t1 + t2;
  }
  c->h[0] += a; c->h[1] += b; c->h[2] += cc; c->h[3] += d; c->h[4] += e; c->h[5] += f; c->h[6] += g; c->h[7] += h;
}
static void sha_init(sha_ctx *c) {
  static const uint32_t i0[8] = {0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
  memcpy(c->h, i0, sizeof i0); c->blen = 0; c->total = 0;
}
static void sha_update(sha_ctx *c, const uint8_t *d, size_t n) {
  c->total += n;
  while (n) {
    size_t k = 64 - c->blen; if (k > n) k = n;
    memcpy(c->buf + c->blen, d, k); c->blen += k; d += k; n -= k;
    if (c->blen == 64) { sha_block(c, c->buf); c->blen = 0; }
  }
}
static void sha_final(sha_ctx *c, uint8_t out[32]) {
  uint64_t bits = c->total * 8;
  uint8_t pad = 0x80; sha_update(c, &pad, 1);
  uint8_t z = 0; while (c->blen != 56) sha_update(c, &z, 1);
  uint8_t l[8]; for (int i = 0; i < 8; i++) l[i] = (uint8_t)(bits >> (56 - 8 * i));
  sha_update(c, l, 8);
  for (int i = 0; i < 8; i++) { out[4*i] = c->h[i] >> 24; out[4*i+1] = c->h[i] >> 16; out[4*i+2] = c->h[i] >> 8; out[4*i+3] = c->h[i]; }
}
void ps4s_sha256(const uint8_t *data, size_t len, uint8_t out[32]) { sha_ctx c; sha_init(&c); sha_update(&c, data, len); sha_final(&c, out); }

void ps4s_hmac(const uint8_t *key, size_t keylen, const uint8_t *const *parts, const size_t *lens, size_t n, uint8_t out[32]) {
  uint8_t k[64] = {0}, ipad[64], opad[64], inner[32];
  if (keylen > 64) ps4s_sha256(key, keylen, k); else memcpy(k, key, keylen);
  for (int i = 0; i < 64; i++) { ipad[i] = k[i] ^ 0x36; opad[i] = k[i] ^ 0x5c; }
  sha_ctx c; sha_init(&c); sha_update(&c, ipad, 64);
  for (size_t i = 0; i < n; i++) sha_update(&c, parts[i], lens[i]);
  sha_final(&c, inner);
  sha_init(&c); sha_update(&c, opad, 64); sha_update(&c, inner, 32); sha_final(&c, out);
}

void ps4s_to_hex(const uint8_t *b, size_t n, char *out) {
  static const char *d = "0123456789abcdef";
  for (size_t i = 0; i < n; i++) { out[2*i] = d[b[i] >> 4]; out[2*i+1] = d[b[i] & 15]; }
  out[2*n] = 0;
}
int ps4s_ct_equal(const char *a, const char *b) {
  size_t la = strlen(a), lb = strlen(b); if (la != lb) return 0;
  uint8_t r = 0; for (size_t i = 0; i < la; i++) r |= (uint8_t)(a[i] ^ b[i]);
  return r == 0;
}

int ps4s_valid_code(const char *c) {
  if (strlen(c) != 8) return 0;
  for (int i = 0; i < 8; i++) if (c[i] < '0' || c[i] > '9') return 0;
  return 1;
}
int ps4s_valid_theme_id(const char *id) {
  size_t n = strlen(id); if (n == 0 || n > 40) return 0;
  for (size_t i = 0; i < n; i++) {
    char ch = id[i];
    int ok = (ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9') || (i > 0 && ch == '-');
    if (!ok) return 0;
  }
  return 1;
}
static int in_list(const char *s, const char *const *l, size_t n) { for (size_t i = 0; i < n; i++) if (strcmp(s, l[i]) == 0) return 1; return 0; }
int ps4s_command_allowed(const char *cmd) {
  static const char *const L[] = {"status","list_themes","put_theme","delete_theme","get_config","set_config","diagnostics"};
  return in_list(cmd, L, 7);
}
int ps4s_config_key_allowed(const char *key) { static const char *const L[] = {"active_theme","show_clock"}; return in_list(key, L, 2); }

static void proof(const char *label, const char *code, const char *cn, const char *sn, uint8_t out[32]) {
  const uint8_t *p[4] = {(const uint8_t *)label, (const uint8_t *)cn, (const uint8_t *)"|", (const uint8_t *)sn};
  size_t l[4] = {strlen(label), strlen(cn), 1, strlen(sn)};
  ps4s_hmac((const uint8_t *)code, strlen(code), p, l, 4, out);
}
void ps4s_client_proof(const char *code, const char *cn, const char *sn, char out[65]) { uint8_t m[32]; proof("pair-client|", code, cn, sn, m); ps4s_to_hex(m, 32, out); }
void ps4s_server_proof(const char *code, const char *cn, const char *sn, char out[65]) { uint8_t m[32]; proof("pair-server|", code, cn, sn, m); ps4s_to_hex(m, 32, out); }
void ps4s_session_key(const char *code, const char *cn, const char *sn, uint8_t out[32]) { proof("session|", code, cn, sn, out); }

void ps4s_frame_mac(const uint8_t key[32], char dir, const char *body, size_t bodylen, char out[65]) {
  const char *label = dir == 'c' ? "c2s|" : "s2c|";
  const uint8_t *p[2] = {(const uint8_t *)label, (const uint8_t *)body}; size_t l[2] = {4, bodylen};
  uint8_t m[32]; ps4s_hmac(key, 32, p, l, 2, m); ps4s_to_hex(m, 32, out);
}
int ps4s_frame_verify(const uint8_t key[32], char dir, const char *body, size_t bodylen, const char *mac_hex) {
  char want[65]; ps4s_frame_mac(key, dir, body, bodylen, want); return ps4s_ct_equal(want, mac_hex);
}
