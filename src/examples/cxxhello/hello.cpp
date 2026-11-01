// C++ program for the Xbox 360 (clang++). Like chello, it calls DbgPrint
// through the SDK headers and stores a heartbeat that `xeedee getmem` can
// read. The counter is a static object, which the C runtime constructs
// before main.
#include <xbox360/xboxkrnl.h>

__attribute__((used, aligned(4))) volatile unsigned int HEARTBEAT = 0x43580000u; // "CX.."

namespace {
template <typename T> class Counter {
public:
    explicit Counter(T start) : value_(start) {
        DbgPrint("cxxhello: counter constructed at %u\n", start);
    }
    T next() { return value_++; }

private:
    T value_;
};

Counter<unsigned int> counter(100);
} // namespace

int main() {
    for (;;) {
        unsigned int value = counter.next();
        DbgPrint("cxxhello: hello from C++ on the xbox 360, built with clang (%u)\n", value);
        HEARTBEAT = 0x43580000u | (value & 0xffff);
        for (volatile int s = 0; s < 4000000; s++) {
        }
    }
}
