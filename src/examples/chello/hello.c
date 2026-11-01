// C program for the Xbox 360 (clang). It calls DbgPrint through the SDK
// headers and stores a memory heartbeat, so execution is observable via
// `xeedee getmem` even without a debug channel.
#include <xbox360/xboxkrnl.h>

__attribute__((used, aligned(4))) volatile unsigned int HEARTBEAT = 0x43480000u; // "CH.."

int main(int argc, char **argv) {
    for (int i = 0; i < argc; i++) {
        DbgPrint("chello: argv[%d] = %s\n", i, argv[i]);
    }
    for (unsigned int i = 0;; i++) {
        DbgPrint("chello: hello from C on the xbox 360, built with clang (%u)\n", i);
        HEARTBEAT = 0x43480000u | (i & 0xffff);
        for (volatile int s = 0; s < 4000000; s++) { }
    }
}
