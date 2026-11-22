# Host-side check (works anywhere with gcc/clang):   make host-test
# PS4 build (needs the OpenOrbis toolchain, UNTESTED here):   OO_PS4_TOOLCHAIN=/path/to/OpenOrbis make ps4
CC ?= cc
.PHONY: host-test ps4 clean

host-test:
	$(CC) -std=c11 -Wall -Wextra -Werror -Iinclude src/proto.c tests/test_proto.c -o /tmp/ps4s_test_proto
	/tmp/ps4s_test_proto

# --- OpenOrbis build: adapted from the toolchain's sample Makefiles; verify flags against your toolchain version. ---
OO ?= $(OO_PS4_TOOLCHAIN)
ps4:
	@test -n "$(OO)" || (echo "Set OO_PS4_TOOLCHAIN to your OpenOrbis toolchain directory" && false)
	clang --target=x86_64-pc-freebsd12-elf -fPIC -funwind-tables -c -isysroot $(OO) -isystem $(OO)/include -Iinclude src/proto.c -o proto.o
	clang --target=x86_64-pc-freebsd12-elf -fPIC -funwind-tables -c -isysroot $(OO) -isystem $(OO)/include -Iinclude src/ps4_main.c -o ps4_main.o
	ld.lld -m elf_x86_64 -pie --script $(OO)/link.x --eh-frame-hdr -L$(OO)/lib $(OO)/lib/crt1.o proto.o ps4_main.o -lc -lkernel -lSceNet -o eboot.elf
	@echo "Linked eboot.elf. Packaging (create-fself / create-gp4 / pkg) is described in README.md."

clean:
	rm -f *.o eboot.elf /tmp/ps4s_test_proto
