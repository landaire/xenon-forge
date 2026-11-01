/*
 * Startup of a title: runs the static constructors, splits the command
 * line of the running executable into arguments and calls main.
 */

#include <xbox360/xboxkrnl.h>

int main(int argc, char **argv);

/* The linker defines the bounds of .init_array. */
typedef void (*CONSTRUCTOR)(void);
extern const CONSTRUCTOR __init_array_start[] __attribute__((weak));
extern const CONSTRUCTOR __init_array_end[] __attribute__((weak));

/* Every argument takes at least one byte and its separator. */
#define MAX_ARGUMENTS (EX_LOADED_COMMAND_LINE_SIZE / 2 + 1)

static CHAR Arguments[EX_LOADED_COMMAND_LINE_SIZE + MAX_ARGUMENTS];
static PCHAR ArgumentVector[MAX_ARGUMENTS + 1];

static int IsSpace(CHAR Character)
{
    return Character == ' ' || Character == '\t';
}

/*
 * Splits CommandLine with the Microsoft C argument rules that Xbox
 * executables use, as the Rust standard library for the console does. The
 * first argument, the executable name, has the simpler quoting rules of
 * executable names.
 */
static int SplitCommandLine(PCSTR CommandLine, SIZE_T Length)
{
    SIZE_T At = 0;
    PCHAR Out = Arguments;
    int Count = 0;
    int Quoted = 0;

    if (Length == 0) {
        ArgumentVector[0] = NULL;
        return 0;
    }

    ArgumentVector[Count++] = Out;
    for (; At < Length; At++) {
        if (CommandLine[At] == '"') {
            Quoted = !Quoted;
        } else if (IsSpace(CommandLine[At]) && !Quoted) {
            break;
        } else {
            *Out++ = CommandLine[At];
        }
    }
    *Out++ = '\0';

    while (At < Length) {
        while (At < Length && IsSpace(CommandLine[At])) {
            At++;
        }
        if (At == Length) {
            break;
        }

        ArgumentVector[Count++] = Out;
        Quoted = 0;
        while (At < Length) {
            if (IsSpace(CommandLine[At]) && !Quoted) {
                break;
            }
            if (CommandLine[At] == '\\') {
                SIZE_T Start = At;
                SIZE_T Backslashes;
                while (At < Length && CommandLine[At] == '\\') {
                    At++;
                }
                Backslashes = At - Start;
                if (At < Length && CommandLine[At] == '"') {
                    for (SIZE_T Index = 0; Index < Backslashes / 2; Index++) {
                        *Out++ = '\\';
                    }
                    if (Backslashes % 2 == 1) {
                        *Out++ = '"';
                        At++;
                    }
                } else {
                    for (SIZE_T Index = 0; Index < Backslashes; Index++) {
                        *Out++ = '\\';
                    }
                }
                continue;
            }
            if (CommandLine[At] == '"' && Quoted && At + 1 < Length && CommandLine[At + 1] == '"') {
                *Out++ = '"';
                At += 2;
                continue;
            }
            if (CommandLine[At] == '"') {
                Quoted = !Quoted;
            } else {
                *Out++ = CommandLine[At];
            }
            At++;
        }
        *Out++ = '\0';
    }

    ArgumentVector[Count] = NULL;
    return Count;
}

int __xbox360_crt_main(void)
{
    PCSTR CommandLine = ExLoadedCommandLine;
    SIZE_T Length = 0;

    for (const CONSTRUCTOR *Constructor = __init_array_start; Constructor < __init_array_end; Constructor++) {
        (*Constructor)();
    }

    while (Length < EX_LOADED_COMMAND_LINE_SIZE && CommandLine[Length] != '\0') {
        Length++;
    }

    return main(SplitCommandLine(CommandLine, Length), ArgumentVector);
}
