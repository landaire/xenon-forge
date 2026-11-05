"""The optional PPC64 compiler distributions and console C compilers."""

load("@prelude//decls:toolchains_common.bzl", "toolchains_common")
load("@prelude//python_bootstrap:python_bootstrap.bzl", "PythonBootstrapToolchainInfo")
load(":rust.bzl", "RustReleaseInfo")

Xbox360ClangInfo = provider(fields = {
    "target_triple": provider_field(str),
    # Links the objects that the compiler produces.
    "linker": provider_field(RunInfo),
})

def _bootstrap_dist_impl(ctx: AnalysisContext) -> list[Provider]:
    if ctx.attrs.host_triple == None:
        fail("the PPC64 toolchain runs only on x86_64 Linux, arm64 macOS and x86_64 Windows hosts; use the default PPC32 toolchain")
    if ctx.attrs.dist == None:
        fail(
            "the nightly toolchain pin has no archive for {}; set ".format(ctx.attrs.host_triple) +
            "xbox360.ppc64_toolchain = source or name an archive with xbox360.ppc64_toolchain_url",
        )
    dist = ctx.attrs.dist[DefaultInfo].default_outputs[0]
    return [
        DefaultInfo(default_output = dist),
        RustReleaseInfo(
            sysroot = dist,
            executable_suffix = ctx.attrs.executable_suffix,
            host_triple = ctx.attrs.host_triple,
        ),
    ]

# A distribution laid out like a Rust release: either a source-built compiler
# or LLVM installation, or the combined archive published by nightly CI.
bootstrap_dist = rule(
    impl = _bootstrap_dist_impl,
    attrs = {
        # None on a host that the prebuilt pin has no archive for.
        "dist": attrs.option(attrs.exec_dep()),
        "executable_suffix": attrs.default_only(attrs.string(default = select({
            "DEFAULT": "",
            "prelude//os:windows": ".exe",
        }))),
        "host_triple": attrs.default_only(attrs.option(attrs.string(), default = select({
            "DEFAULT": None,
            "prelude//os:linux": select({
                "DEFAULT": None,
                "prelude//cpu:x86_64": "x86_64-unknown-linux-gnu",
            }),
            "prelude//os:macos": select({
                "DEFAULT": None,
                "prelude//cpu:arm64": "aarch64-apple-darwin",
            }),
            "prelude//os:windows": select({
                "DEFAULT": None,
                "prelude//cpu:x86_64": "x86_64-pc-windows-msvc",
            }),
        }))),
    },
)

def prebuilt_toolchain_archive(name: str, url: str, sha256: str, strip_prefix: str):
    """Downloads a combined Rust and LLVM distribution published by nightly CI."""
    if not url or not sha256 or not strip_prefix:
        fail(
            "the prebuilt PPC64 toolchain is not pinned; use the generated " +
            "nightly-toolchain branch or set xbox360.ppc64_toolchain_url, " +
            "xbox360.ppc64_toolchain_sha256 and xbox360.ppc64_toolchain_strip_prefix",
        )
    native.http_archive(
        name = name,
        sha256 = sha256,
        strip_prefix = strip_prefix,
        urls = [url],
    )

def _path_clang_impl(ctx: AnalysisContext) -> list[Provider]:
    return [
        DefaultInfo(),
        RunInfo(args = cmd_args(
            ctx.attrs._python_bootstrap_toolchain[PythonBootstrapToolchainInfo].interpreter,
            ctx.attrs._runner,
        )),
    ]

# clang from the PATH of the Buck2 daemon.
path_clang = rule(
    impl = _path_clang_impl,
    attrs = {
        "_python_bootstrap_toolchain": toolchains_common.python_bootstrap(),
        "_runner": attrs.source(default = "xbox360//buck:run_clang.py"),
    },
)

def _xbox360_clang_impl(ctx: AnalysisContext) -> list[Provider]:
    return [
        DefaultInfo(),
        ctx.attrs.tool[RunInfo],
        Xbox360ClangInfo(
            target_triple = ctx.attrs.target_triple,
            linker = ctx.attrs.linker[RunInfo],
        ),
    ]

# The C compiler and linker of console titles.
xbox360_clang = rule(
    impl = _xbox360_clang_impl,
    attrs = {
        "linker": attrs.exec_dep(providers = [RunInfo]),
        "target_triple": attrs.string(),
        "tool": attrs.exec_dep(providers = [RunInfo]),
    },
    is_toolchain_rule = True,
)
