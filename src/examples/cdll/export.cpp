// XBOX360_EXPORT also works in C++. Ordinal 2 multiplies two numbers.
#include <xbox360/xboxkrnl.h>

namespace {

ULONG Multiply(ULONG Left, ULONG Right) {
    return Left * Right;
}

} // namespace

XBOX360_EXPORT(2, Multiply);
