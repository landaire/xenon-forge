/*
 * Entry of a DLL or system module: `_start` runs
 * `BOOL DllMain(HANDLE Module, DWORD Reason, PVOID Reserved)`, with
 * DLL_PROCESS_ATTACH when the module is loaded and DLL_PROCESS_DETACH when
 * it is unloaded. A FALSE result on attach fails the load.
 */

#include "entry.inc"

	xbox360_entry _start, DllMain
