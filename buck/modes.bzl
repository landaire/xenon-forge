"""Build modes and code generations as buck2 configuration modifiers.

A project enables them by calling `xbox360_modifiers()` from its root
PACKAGE file:

    load("@xbox360//buck:modes.bzl", "xbox360_modifiers")

    xbox360_modifiers()

Then `buck2 build -m release //title` builds in release mode, and `-m ppc64-ilp32`
with the PPC64 ILP32 code generation. Without a modifier the mode is
`xbox360.mode` of the root buckconfig, which the mode files set
(`buck2 build @xbox360//mode/release`), and dev without it.
"""

load("@prelude//cfg/modifier:cfg_constructor.bzl", "cfg_constructor_post_constraint_analysis", "cfg_constructor_pre_constraint_analysis")
load("@prelude//cfg/modifier:common.bzl", "MODIFIER_METADATA_KEY")

XBOX360_MODIFIER_ALIASES = {
    "dev": "xbox360//platforms:mode[dev]",
    "ppc32": "xbox360//platforms:codegen[ppc32]",
    "ppc64-ilp32": "xbox360//platforms:codegen[ppc64-ilp32]",
    "release": "xbox360//platforms:mode[release]",
    "release-no-symbols": "xbox360//platforms:mode[release-no-symbols]",
}

def xbox360_modifiers(aliases: dict[str, str] = {}):
    """Sets the configuration constructor of the project with the modifier
    aliases of this cell and `aliases`, which take precedence."""
    native.set_cfg_constructor(
        stage0 = cfg_constructor_pre_constraint_analysis,
        stage1 = cfg_constructor_post_constraint_analysis,
        key = MODIFIER_METADATA_KEY,
        aliases = struct(**(XBOX360_MODIFIER_ALIASES | aliases)),
        extra_data = struct(),
    )
