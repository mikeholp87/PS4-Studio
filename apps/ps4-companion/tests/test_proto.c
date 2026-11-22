/* Host test: gcc -Iinclude src/proto.c tests/test_proto.c -o /tmp/test_proto && /tmp/test_proto
 * Vectors in tests/vectors.json were computed independently with Python's hmac module; the Rust
 * implementation is checked against the same constants (crates/core companion_proto tests). */
#include "proto.h"
#include <stdio.h>
#include <string.h>

static int fails = 0;
#define CHECK(c) do { if (!(c)) { printf("FAIL line %d: %s\n", __LINE__, #c); fails++; } } while (0)

int main(void) {
  char hex[65]; uint8_t m[32];
  /* RFC 4231 test case 2 */
  const uint8_t *p[1] = {(const uint8_t *)"what do ya want for nothing?"}; size_t l[1] = {28};
  ps4s_hmac((const uint8_t *)"Jefe", 4, p, l, 1, m); ps4s_to_hex(m, 32, hex);
  CHECK(strcmp(hex, "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843") == 0);
  /* SHA-256("abc") */
  ps4s_sha256((const uint8_t *)"abc", 3, m); ps4s_to_hex(m, 32, hex);
  CHECK(strcmp(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad") == 0);

  const char *code = "24681357", *cn = "00112233445566778899aabbccddeeff", *sn = "ffeeddccbbaa99887766554433221100";
  ps4s_client_proof(code, cn, sn, hex);
  CHECK(strcmp(hex, "d748a49f7e847a7b660590adffc4d37c867bf2a051eebf59ed860eb0f85e7dd4") == 0);
  ps4s_server_proof(code, cn, sn, hex);
  CHECK(strcmp(hex, "acbfc00f20aef725810b4552b3dbe54ca8d4ea2b291023dfda16db9679a6ba5c") == 0);
  uint8_t key[32]; ps4s_session_key(code, cn, sn, key); ps4s_to_hex(key, 32, hex);
  CHECK(strcmp(hex, "194ae16320cf7119bb677d44bdcc7d13dad867acb1d0baf487f7d87a94e23fe9") == 0);
  const char *body = "{\"seq\":1,\"cmd\":\"status\",\"args\":{}}";
  ps4s_frame_mac(key, 'c', body, strlen(body), hex);
  CHECK(strcmp(hex, "0e81f4f4f4658f73aa2005d268891006a4c67012f399f47fa4e755bb408d43d4") == 0);
  CHECK(ps4s_frame_verify(key, 'c', body, strlen(body), hex));
  CHECK(!ps4s_frame_verify(key, 's', body, strlen(body), hex)); /* reflection */
  CHECK(!ps4s_frame_verify(key, 'c', body, strlen(body) - 1, hex)); /* tamper */

  /* long key path (>64 bytes) must hash the key first */
  uint8_t big[100]; memset(big, 0xaa, sizeof big);
  const uint8_t *q[1] = {(const uint8_t *)"x"}; size_t ql[1] = {1};
  ps4s_hmac(big, sizeof big, q, ql, 1, m); ps4s_to_hex(m, 32, hex); CHECK(strlen(hex) == 64);

  CHECK(ps4s_valid_code("00000000") && !ps4s_valid_code("1234567") && !ps4s_valid_code("1234567a") && !ps4s_valid_code("123456789"));
  CHECK(ps4s_valid_theme_id("my-theme-2") && !ps4s_valid_theme_id("../x") && !ps4s_valid_theme_id("-a") && !ps4s_valid_theme_id("A") && !ps4s_valid_theme_id(""));
  CHECK(ps4s_command_allowed("status") && ps4s_command_allowed("put_theme"));
  CHECK(!ps4s_command_allowed("exec") && !ps4s_command_allowed("shell") && !ps4s_command_allowed("read_file") && !ps4s_command_allowed("status "));
  CHECK(ps4s_config_key_allowed("show_clock") && !ps4s_config_key_allowed("/etc/passwd"));
  CHECK(ps4s_ct_equal("abc", "abc") && !ps4s_ct_equal("abc", "abd") && !ps4s_ct_equal("abc", "ab"));

  printf(fails ? "%d FAILED\n" : "all protocol checks passed\n", fails);
  return fails != 0;
}
