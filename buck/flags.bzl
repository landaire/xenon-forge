"""Code generation settings for Xbox 360 code, by build mode.

The mode is the `xbox360//platforms:mode` constraint, which the mode files
(`@xbox360//mode/release`) set. dev compiles without optimizations; the
release modes optimize. Debug information stays in the ELF that a debugger
loads; xex-link copies only loaded sections into the XEX.
"""

_RELEASE = "xbox360//platforms:mode[release]"
_RELEASE_NO_SYMBOLS = "xbox360//platforms:mode[release-no-symbols]"

# Size matters more than speed for code that shares the console's memory
# with the running title. The standard library crates always use it, as
# the Rust release ships them optimized.
XBOX360_OPT_LEVEL = "z"

def xbox360_rustc_flags():
    """rustc flags of console crates."""
    return ["-Ccodegen-units=1"] + select({
        _RELEASE: ["-Copt-level=" + XBOX360_OPT_LEVEL, "-Cdebuginfo=full"],
        _RELEASE_NO_SYMBOLS: ["-Copt-level=" + XBOX360_OPT_LEVEL, "-Cdebuginfo=none"],
        "DEFAULT": ["-Copt-level=0", "-Cdebuginfo=full"],
    })

def xbox360_c_flags():
    """clang flags of console C and C++ sources."""
    return select({
        _RELEASE: ["-O2", "-g"],
        _RELEASE_NO_SYMBOLS: ["-O2"],
        "DEFAULT": ["-O0", "-g"],
    })

# The standard library crates, as in the Rust release: lines and inlined
# functions without types or variables.
XBOX360_SYSROOT_DEBUGINFO = "-Cdebuginfo=limited"
