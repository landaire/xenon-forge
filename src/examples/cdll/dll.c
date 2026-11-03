// C DLL for the Xbox 360 (clang). `//src/sdk:crt-dll` calls DllMain when
// the module loads and unloads. Ordinal 1 adds two numbers; another module
// finds it with XexGetProcedureAddress(Module, 1, &Address).
#include <xbox360/xboxkrnl.h>

static ULONG Add(ULONG Left, ULONG Right) {
    return Left + Right;
}

XBOX360_EXPORT(1, Add);

BOOL DllMain(HANDLE Module, DWORD Reason, PVOID Reserved) {
    (void)Module;
    (void)Reserved;
    if (Reason == DLL_PROCESS_ATTACH) {
        DbgPrint("cdll: loaded\n");
    }
    return TRUE;
}
