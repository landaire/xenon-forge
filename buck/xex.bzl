"""Xbox 360 titles: a Rust, C or C++ executable converted to a XEX by xex-link.

`xex_core`, `xex_std` and `xex_std_system` convert the image of any
executable rule.

`xbox360_executable_core`, `xbox360_executable_std` and
`xbox360_executable_std_system` compile a binary crate as one fat LTO code
generation unit, and rustc writes that unit as assembly along with the image
it links. The prelude's `rust_binary` has
no such output: its `[asm]` subtarget compiles without linking, which
skips LTO. `xex_from_assembly` turns that assembly into a XEX. With
`rewrite`, xbox360-asm-rewrite replaces the callee-saved register saves
and restores in prologues and epilogues with calls to shared helper
routines, and an audit of the linked image's disassembly fails the build
if a rewritable sequence remains. rustc assembles the result with the
integrated assembler of its LLVM, and rust-lld links it with the code that
LTO leaves out (compiler_builtins). The link mirrors the one rustc runs;
the build fails if the mirror, applied to the object rustc linked, does
not give rustc's image.

`xbox360_c_title` compiles C and C++ sources with clang and converts the
image.

Every rule fails the build if the image holds a reservation that the
kernel loader reports as unprotected.

Every rule is runnable: `buck2 run` uploads the XEX through xeedee and
launches it. `[debug]` uploads it, starts openxbdm-kd, launches the title,
and opens GDB. `[debug-server]` leaves the GDB server waiting for an
external debugger, and `[vscode]` emits a ready-to-open VS Code workspace.
`[elf]` is the image with symbols and debug information, which xex-link
leaves out of the XEX. `[gdb]` and `[lldb]` are command files that load it
into a debugger.
"""

load("@prelude//decls:toolchains_common.bzl", "toolchains_common")
load("@prelude//linking:link_info.bzl", "LinkStrategy")
load("@prelude//python_bootstrap:python_bootstrap.bzl", "PythonBootstrapToolchainInfo")
load("@prelude//rust:build_params.bzl", "MetadataKind")
load("@prelude//rust:link_info.bzl", "RustLinkInfo", "RustProcMacroInfo", "RustProcMacroPlugin", "get_available_proc_macros")
load("@prelude//rust:rust_toolchain.bzl", "RustToolchainInfo")
load(":custom_toolchain.bzl", "Xbox360ClangInfo")
load(":flags.bzl", "xbox360_c_flags")
load(":platforms.bzl", "xbox360_core_transition", "xbox360_std_system_transition", "xbox360_std_transition")

Xbox360ExecutableInfo = provider(fields = {
    # The LTO code generation unit as assembly.
    "assembly": provider_field(Artifact),
    # The `compiler_builtins` rlib, which LTO leaves out.
    "compiler_builtins": provider_field(Artifact),
    # The image rustc links.
    "image": provider_field(Artifact),
    "link_script": provider_field(Artifact),
    # The LTO code generation unit as the object rustc links.
    "object": provider_field(Artifact),
})

_TOOL_ATTRS = {
    "_asm_rewrite": attrs.exec_dep(
        providers = [RunInfo],
        default = "xbox360//src/crates/xbox360-asm-rewrite:xbox360-asm-rewrite",
    ),
    "_console_kd_target": attrs.string(default = read_root_config("xbox360", "console_kd_target") or ""),
    "_console_xbdm": attrs.string(default = read_root_config("xbox360", "console_xbdm") or ""),
    "_deploy_directory": attrs.string(default = read_root_config("xbox360", "deploy_directory") or ""),
    "_debug_registers": attrs.string(default = select({
        "//platforms:xbox-ppc64-ilp32": "ilp32",
        "DEFAULT": "32",
    })),
    "_gdb_listen": attrs.string(default = read_root_config("xbox360", "gdb_listen") or "127.0.0.1:2159"),
    "_gdb_tool": attrs.string(default = read_root_config("xbox360", "gdb_tool") or "gdb"),
    "_kd_tool": attrs.string(default = read_root_config("xbox360", "kd_tool") or ""),
    "_kd_tool_fallback": attrs.exec_dep(
        providers = [RunInfo],
        default = "xbox360//buck:openxbdm-kd",
    ),
    "_python_bootstrap_toolchain": toolchains_common.python_bootstrap(),
    "_run_title": attrs.source(default = "xbox360//buck:run_title.py"),
    "_write_stdout": attrs.source(default = "xbox360//buck:write_stdout.py"),
    "_write_vscode_workspace": attrs.source(default = "xbox360//buck:write_vscode_workspace.py"),
    "_xbdm_tool": attrs.string(default = read_root_config("xbox360", "xbdm_tool") or ""),
    "_xbdm_tool_fallback": attrs.exec_dep(
        providers = [RunInfo],
        default = "xbox360//buck:xeedee",
    ),
    "_xex_link": attrs.exec_dep(providers = [RunInfo], default = "xbox360//src/crates/xex-link:xex-link"),
}

def _run_title_command(
        ctx: AnalysisContext,
        xex: Artifact,
        debug_mode: str | None = None,
        gdb_commands: Artifact | None = None) -> cmd_args:
    command = cmd_args(
        ctx.attrs._python_bootstrap_toolchain[PythonBootstrapToolchainInfo].interpreter,
        ctx.attrs._run_title,
        "--fallback-xbdm-tool",
        ctx.attrs._xbdm_tool_fallback[RunInfo],
        "--fallback-kd-tool",
        ctx.attrs._kd_tool_fallback[RunInfo],
        "--gdb",
        ctx.attrs._gdb_tool,
        "--listen",
        ctx.attrs._gdb_listen,
        "--registers",
        ctx.attrs._debug_registers,
    )
    if ctx.attrs._xbdm_tool:
        command.add("--xbdm-tool", ctx.attrs._xbdm_tool)
    if ctx.attrs._kd_tool:
        command.add("--kd-tool", ctx.attrs._kd_tool)
    if ctx.attrs._console_xbdm:
        command.add("--host", ctx.attrs._console_xbdm)
    if ctx.attrs._deploy_directory:
        command.add("--directory", ctx.attrs._deploy_directory)
    if ctx.attrs._console_kd_target:
        command.add("--kd-target", ctx.attrs._console_kd_target)
    if debug_mode == "gdb":
        gdb_commands or fail("GDB debugging requires a command file")
        command.add("--debug", "--gdb-commands", gdb_commands)
    elif debug_mode == "server":
        command.add("--debug-server")
    elif debug_mode != None:
        fail("unknown debug mode {}".format(debug_mode))
    command.add(xex)
    return command

# Files that a XEX embeds as named resources, which the title finds with
# `XexGetModuleSection`.
XexResourcesInfo = provider(fields = {
    # Resource name (1 to 8 bytes) to the file.
    "files": provider_field(dict[str, Artifact]),
})

def _xex_resources_impl(ctx: AnalysisContext) -> list[Provider]:
    return [DefaultInfo(other_outputs = ctx.attrs.files.values()), XexResourcesInfo(files = ctx.attrs.files)]

# Resources for the `resources` of the XEX rules and `xbox360_title`: a map
# of resource names to files or targets.
xex_resources = rule(
    impl = _xex_resources_impl,
    attrs = {"files": attrs.dict(attrs.string(), attrs.source())},
)

def _write_stdout(ctx: AnalysisContext, output: Artifact, command: cmd_args, category: str):
    ctx.actions.run(
        cmd_args(
            ctx.attrs._python_bootstrap_toolchain[PythonBootstrapToolchainInfo].interpreter,
            ctx.attrs._write_stdout,
            output.as_output(),
            command,
        ),
        category = category,
    )

def _check_reservations(ctx: AnalysisContext, image: Artifact) -> Artifact:
    reservations = ctx.actions.declare_output("reservations")
    _write_stdout(
        ctx,
        reservations,
        cmd_args(ctx.attrs._asm_rewrite[RunInfo], "reservations", image),
        category = "reservations",
    )
    return reservations

def _xex_link(ctx: AnalysisContext, image: Artifact, checks: list[Artifact]) -> Artifact:
    xex = ctx.actions.declare_output(ctx.attrs.out or ctx.label.name + ".xex")
    cmd = cmd_args(
        ctx.attrs._xex_link[RunInfo],
        image,
        xex.as_output(),
        # The XEX exists only if the checks pass.
        hidden = checks,
    )
    if ctx.attrs.manifest:
        cmd.add("--manifest", ctx.attrs.manifest)
    if ctx.attrs.resources:
        for name, src in ctx.attrs.resources[XexResourcesInfo].files.items():
            cmd.add("--resource", cmd_args(name, "=", src, delimiter = ""))
    ctx.actions.run(cmd, category = "xex_link")
    return xex

def _generated_link_script(ctx: AnalysisContext, manifest: Artifact | None) -> Artifact:
    """The GNU ld script that xex-link derives from `manifest`."""
    script = ctx.actions.declare_output("link.x")
    cmd = cmd_args(ctx.attrs._xex_link[RunInfo], "link-script", script.as_output())
    if manifest:
        cmd.add("--manifest", manifest)
    ctx.actions.run(cmd, category = "link_script")
    return script

def _xex_link_script_impl(ctx: AnalysisContext) -> list[Provider]:
    return [DefaultInfo(default_output = _generated_link_script(ctx, ctx.attrs.manifest))]

# The GNU ld script of a module for its xex-link manifest: code one page
# above the load address and each kind of section on its own pages.
xex_link_script = rule(
    impl = _xex_link_script_impl,
    attrs = {
        "manifest": attrs.option(attrs.source(), default = None),
        "_xex_link": attrs.exec_dep(providers = [RunInfo], default = "xbox360//src/crates/xex-link:xex-link"),
    },
)

def _debugger_files(
        ctx: AnalysisContext,
        xex: Artifact,
        elf: Artifact,
        library: Dependency | None,
        rust_project: Artifact | None) -> dict[str, list[Provider]]:
    """Debugger artifacts and runnable debugger subtargets.

    The command files load `elf` and, with `library`
    (`xbox360//sysroot:library`), find the standard library source. Paths are
    relative to the project root, as are the source paths in the debug
    information, so the debugger runs there. The VS Code workspace links
    `rust_project` for Rust Analyzer when one is present.
    """
    gdb = [cmd_args("file ", elf, delimiter = "")]
    lldb = [cmd_args("target create ", elf, delimiter = "")]
    outputs = [elf]

    if library:
        source = library[DefaultInfo].default_outputs[0]

        # rustc records the library sources under the package of the
        # sysroot crates, without the leading `./` of the remapped prefix.
        recorded = cmd_args(library.label.path, "/library", delimiter = "", replace_regex = ("^\\./", ""))
        gdb.append(cmd_args("set substitute-path ", recorded, " ", source, delimiter = ""))
        lldb.append(cmd_args("settings set target.source-map ", recorded, " ", source, delimiter = ""))
        outputs.append(source)

    files = {}
    gdb_script = None
    for name, lines in [("gdb", gdb), ("lldb", lldb)]:
        script, _ = ctx.actions.write(name + ".commands", cmd_args(lines, "", delimiter = "\n"), allow_args = True)
        files[name] = [DefaultInfo(default_output = script, other_outputs = outputs)]
        if name == "gdb":
            gdb_script = script

    files["debug"] = [
        DefaultInfo(default_output = xex, other_outputs = outputs + [gdb_script]),
        RunInfo(args = _run_title_command(ctx, xex, "gdb", gdb_script)),
    ]
    files["debug-server"] = [
        DefaultInfo(default_output = xex, other_outputs = outputs),
        RunInfo(args = _run_title_command(ctx, xex, "server")),
    ]

    workspace = ctx.actions.declare_output(ctx.label.name + ".code-workspace")
    workspace_command = cmd_args(
        ctx.attrs._python_bootstrap_toolchain[PythonBootstrapToolchainInfo].interpreter,
        ctx.attrs._write_vscode_workspace,
        "--output",
        workspace.as_output(),
        "--target",
        str(ctx.label.raw_target()),
        "--elf",
        elf,
        "--gdb-commands",
        gdb_script,
        "--gdb",
        ctx.attrs._gdb_tool,
        "--listen",
        ctx.attrs._gdb_listen,
        "--architecture",
        "powerpc:common64" if ctx.attrs._debug_registers == "64" else "powerpc:common",
    )
    if rust_project:
        workspace_command.add("--rust-project", rust_project)
    ctx.actions.run(workspace_command, category = "vscode_workspace")
    files["vscode"] = [
        DefaultInfo(default_output = workspace, other_outputs = outputs + [gdb_script]),
    ]
    return files

def _xex_impl(ctx: AnalysisContext) -> list[Provider]:
    elf = ctx.attrs.elf[DefaultInfo].default_outputs[0]
    reservations = _check_reservations(ctx, elf)
    xex = _xex_link(ctx, elf, [reservations])
    return [
        DefaultInfo(default_output = xex, sub_targets = {
            "elf": [DefaultInfo(default_output = elf)],
            "reservations": [DefaultInfo(default_output = reservations)],
        } | _debugger_files(ctx, xex, elf, ctx.attrs._sysroot_library, ctx.attrs.rust_project)),
        RunInfo(args = _run_title_command(ctx, xex)),
    ]

_XEX_ATTRS = {
    # See xex-link's documentation for the format; without one, xex-link's
    # defaults apply.
    "manifest": attrs.option(attrs.source(), default = None),
    "out": attrs.option(attrs.string(), default = None),
    # An `xex_resources` target: the files that the image embeds by
    # resource name.
    "resources": attrs.option(attrs.dep(providers = [XexResourcesInfo]), default = None),
    # Cargo project loaded by rust-analyzer in the generated VS Code workspace.
    "rust_project": attrs.option(attrs.source(), default = None),
} | _TOOL_ATTRS

# The standard library source, for the debugger command files of Rust
# titles.
_SYSROOT_LIBRARY_ATTRS = {
    "_sysroot_library": attrs.dep(default = "xbox360//sysroot:library"),
}

# The rule configures itself, and so the executable, for the console. The
# core variant provides core, alloc and compiler_builtins; the std variant
# also std, and the std_system variant the std of system modules, whose
# threads start in the system process and whose sockets open as the system.
xex_core = rule(impl = _xex_impl, attrs = _XEX_ATTRS | _SYSROOT_LIBRARY_ATTRS | {"elf": attrs.dep()}, cfg = xbox360_core_transition)

xex_std = rule(impl = _xex_impl, attrs = _XEX_ATTRS | _SYSROOT_LIBRARY_ATTRS | {"elf": attrs.dep()}, cfg = xbox360_std_transition)

xex_std_system = rule(impl = _xex_impl, attrs = _XEX_ATTRS | _SYSROOT_LIBRARY_ATTRS | {"elf": attrs.dep()}, cfg = xbox360_std_system_transition)

# No C++ runtime exists for the console, so C++ sources compile without
# exceptions, RTTI and the guards of function-local statics.
_CXX_EXTENSIONS = (".cc", ".cpp", ".cxx")
_CXX_FLAGS = ["-fno-exceptions", "-fno-rtti", "-fno-threadsafe-statics"]

# The headers and objects of an `xbox360_c_library` and its dependencies.
Xbox360CLibraryInfo = provider(fields = {
    # Directories that sources include headers from.
    "includes": provider_field(list[Artifact]),
    "objects": provider_field(list[Artifact]),
})

def _c_dependencies(deps: list[Dependency]) -> (list[Artifact], list[Artifact]):
    """The include directories and objects of `deps`, each once."""
    includes = []
    objects = []
    for dep in deps:
        library = dep[Xbox360CLibraryInfo]
        includes.extend([include for include in library.includes if include not in includes])
        objects.extend([obj for obj in library.objects if obj not in objects])
    return includes, objects

def _compile(ctx: AnalysisContext, src: Artifact, includes: list[Artifact], private_headers: list[Artifact] = []) -> Artifact:
    """Compiles a C, C++ or assembly source of a title or library."""
    is_cxx = any([src.short_path.endswith(ext) for ext in _CXX_EXTENSIONS])
    obj = ctx.actions.declare_output("objects", src.short_path + ".o")
    ctx.actions.run(
        cmd_args(
            ctx.attrs._clang[RunInfo],
            "--target=" + ctx.attrs._clang[Xbox360ClangInfo].target_triple,
            # No C library exists for the console. C++ compiles as hosted,
            # where `main` is not mangled, so the C runtime can call it.
            [] if is_cxx else "-ffreestanding",
            "-fno-builtin",
            # Optimization and debug information of the build mode. Debug
            # information stays in the ELF that a debugger loads.
            ctx.attrs._mode_flags,
            [cmd_args(include, format = "-I{}") for include in includes],
            _CXX_FLAGS if is_cxx else [],
            ctx.attrs.compiler_flags,
            ctx.attrs.cxx_compiler_flags if is_cxx else [],
            "-c",
            src,
            "-o",
            obj.as_output(),
            # Included from the directory of `src`.
            hidden = private_headers,
        ),
        category = "cc",
        identifier = src.short_path,
    )
    return obj

def _xbox360_c_library_impl(ctx: AnalysisContext) -> list[Provider]:
    includes, dependency_objects = _c_dependencies(ctx.attrs.deps)
    if ctx.attrs.headers:
        headers = {}
        for header in ctx.attrs.headers:
            path = header.short_path
            if not path.startswith(ctx.attrs.header_root + "/"):
                fail("header {} is not under {}".format(path, ctx.attrs.header_root))
            headers[path.removeprefix(ctx.attrs.header_root + "/")] = header
        includes = [ctx.actions.symlinked_dir("include", headers)] + includes
    objects = [_compile(ctx, src, includes, ctx.attrs.private_headers) for src in ctx.attrs.srcs]
    return [
        DefaultInfo(default_outputs = includes + objects),
        Xbox360CLibraryInfo(includes = includes, objects = objects + dependency_objects),
    ]

_C_ATTRS = {
    # After the flags of the build mode, so they can override them.
    "compiler_flags": attrs.list(attrs.string(), default = []),
    "cxx_compiler_flags": attrs.list(attrs.string(), default = []),
    "srcs": attrs.list(attrs.source(), default = []),
    "_mode_flags": attrs.list(attrs.string(), default = xbox360_c_flags()),
    "_clang": attrs.toolchain_dep(
        providers = [RunInfo, Xbox360ClangInfo],
        default = "toolchains//:clang",
    ),
}

# Headers and C, C++ or assembly sources for C and C++ titles. `headers`
# are included relative to `header_root`, a directory of the package.
# `private_headers` are only for `srcs`, which include them from their own
# directory. `deps` are other libraries, whose headers `srcs` include. The
# objects of `srcs` and `deps` link into each title that depends on the
# library; with --gc-sections, only the sections the title reaches stay.
xbox360_c_library = rule(
    impl = _xbox360_c_library_impl,
    attrs = _C_ATTRS | {
        "deps": attrs.list(attrs.dep(providers = [Xbox360CLibraryInfo]), default = []),
        "header_root": attrs.string(default = ""),
        "headers": attrs.list(attrs.source(), default = []),
        "private_headers": attrs.list(attrs.source(), default = []),
    },
)

def _xbox360_c_title_impl(ctx: AnalysisContext) -> list[Provider]:
    includes, library_objects = _c_dependencies(ctx.attrs.deps)
    objects = [_compile(ctx, src, includes) for src in ctx.attrs.srcs] + library_objects
    link_script = ctx.attrs.link_script or _generated_link_script(ctx, ctx.attrs.manifest)

    image = ctx.actions.declare_output("image")
    ctx.actions.run(
        cmd_args(
            ctx.attrs._clang[Xbox360ClangInfo].linker,
            "-flavor",
            "gnu",
            objects,
            "--gc-sections",
            # No dynamic loader protects a RELRO segment; without it
            # .init_array shares the data segment.
            "-z",
            "norelro",
            cmd_args(link_script, format = "-T{}"),
            "--no-rosegment",
            "-o",
            image.as_output(),
        ),
        category = "link",
    )

    reservations = _check_reservations(ctx, image)
    xex = _xex_link(ctx, image, [reservations])
    return [
        DefaultInfo(default_output = xex, sub_targets = {
            "elf": [DefaultInfo(default_output = image)],
            "reservations": [DefaultInfo(default_output = reservations)],
        } | _debugger_files(ctx, xex, image, None, ctx.attrs.rust_project)),
        RunInfo(args = _run_title_command(ctx, xex)),
    ]

# A XEX from C and C++ sources. The configured clang compiles them for
# big-endian PowerPC without a C or C++ library, and its linker links the
# objects with --gc-sections and the GNU ld script `link_script`, by
# default the one xex-link derives from `manifest`. `compiler_flags` apply
# to every source, `cxx_compiler_flags` only to C++ sources (.cc, .cpp,
# .cxx); they follow the flags of the build mode. `deps` are
# `xbox360_c_library` targets: a title lists `xbox360//src/sdk:crt`, which
# defines `_start` and calls `main`, and brings the Xbox 360 headers and
# import thunks; a DLL lists `xbox360//src/sdk:crt-dll`, which calls
# `DllMain`.
xbox360_c_title = rule(
    impl = _xbox360_c_title_impl,
    attrs = _XEX_ATTRS | _C_ATTRS | {
        "deps": attrs.list(attrs.dep(providers = [Xbox360CLibraryInfo]), default = []),
        "link_script": attrs.option(attrs.source(), default = None),
    },
)

def _rlib(dep: Dependency) -> Artifact:
    # Executables for the console link statically (no position-independent
    # code), as with the target specification's relocation model. The
    # `link` rlibs hold the code; `full` ones only metadata.
    return dep[RustLinkInfo].strategies[LinkStrategy("static")].outputs[MetadataKind("link")]

def _dependency_dirs(dep: Dependency) -> cmd_args:
    """`-Ldependency` for the directory of each of `dep`'s transitive rlibs."""
    transitive = dep[RustLinkInfo].strategies[LinkStrategy("static")].transitive_deps[MetadataKind("link")]
    return cmd_args(transitive.project_as_args("artifacts_args"), format = "-Ldependency={}", parent = 1)

def _proc_macro_dirs(ctx: AnalysisContext, deps: list[Dependency]) -> cmd_args:
    """`-Ldependency` for the directory of each proc macro that the transitive
    rlibs use: rustc loads a crate's proc macros to read its metadata."""
    available = get_available_proc_macros(ctx)
    labels = {}
    for dep in deps:
        markers = dep[RustLinkInfo].strategies[LinkStrategy("static")].transitive_proc_macro_deps
        for marker in markers:
            labels[marker] = None
    dylibs = [available[label][RustProcMacroInfo].dylib for label in sorted(labels)]
    return cmd_args(dylibs, format = "-Ldependency={}", parent = 1)

def _extern(dep: Dependency, options: str = "") -> cmd_args:
    crate = dep[RustLinkInfo].crate.simple
    return cmd_args(_rlib(dep), format = "--extern=" + options + crate + "={}")

def _check_rustc_flags(flags: list[str]):
    """Fails on flags that change rustc's link: `xex_from_assembly` repeats
    that link with fixed arguments and would no longer reproduce it."""
    for index, flag in enumerate(flags):
        option = flags[index + 1] if flag == "-C" and index + 1 < len(flags) else flag.removeprefix("-C")
        if option.startswith("link") or flag.startswith("-l"):
            fail("rustc_flags may not change the link: `{}`".format(option if option.startswith("link") else flag))

def _xbox360_executable_impl(ctx: AnalysisContext) -> list[Provider]:
    toolchain = ctx.attrs._rust_toolchain[RustToolchainInfo]
    _check_rustc_flags(ctx.attrs.rustc_flags)
    sysroot = toolchain.explicit_sysroot_deps
    compiler_builtins = [dep for dep in sysroot.others if dep[RustLinkInfo].crate.simple == "compiler_builtins"]
    len(compiler_builtins) == 1 or fail("the toolchain provides no compiler_builtins")

    # The options the prelude passes for the standard library crates.
    # `force` links std even into a crate that does not name it.
    sysroot_externs = [_extern(sysroot.core, "noprelude,nounused:")]
    if sysroot.std:
        sysroot_externs.append(_extern(sysroot.std, "noprelude,nounused,force:"))
    if sysroot.panic_abort:
        sysroot_externs.append(_extern(sysroot.panic_abort, "noprelude,nounused:"))
    sysroot_externs.extend([_extern(dep, "noprelude,nounused:") for dep in sysroot.others])
    sysroot_crates = [dep for dep in [sysroot.core, sysroot.std, sysroot.panic_abort] if dep] + sysroot.others

    srcs = ctx.actions.symlinked_dir("srcs", {src.short_path: src for src in ctx.attrs.srcs})
    empty_sysroot = ctx.actions.symlinked_dir("sysroot", {})
    assembly = ctx.actions.declare_output("{}.s".format(ctx.attrs.crate))
    obj = ctx.actions.declare_output("{}.o".format(ctx.attrs.crate))
    image = ctx.actions.declare_output(ctx.attrs.crate)

    ctx.actions.run(
        cmd_args(
            toolchain.compiler,
            cmd_args(srcs, format = "{}/" + ctx.attrs.crate_root),
            "--crate-name=" + ctx.attrs.crate,
            "--crate-type=bin",
            "--edition=" + ctx.attrs.edition,
            "--target=" + toolchain.rustc_target_triple,
            # Only the crates passed with --extern.
            cmd_args(empty_sysroot, format = "--sysroot={}"),
            ['--cfg=feature="{}"'.format(feature) for feature in ctx.attrs.features],
            [_extern(dep) for dep in ctx.attrs.deps],
            sysroot_externs,
            [_dependency_dirs(dep) for dep in ctx.attrs.deps + sysroot_crates],
            _proc_macro_dirs(ctx, ctx.attrs.deps),
            cmd_args(srcs, format = "--remap-path-prefix={}/=" + ctx.label.package + "/"),
            # Debug information names files relative to the project root,
            # as with the prelude's Rust rules.
            "-Zremap-cwd-prefix=.",
            cmd_args(ctx.attrs._rust_lld[RunInfo], format = "-Clinker={}"),
            cmd_args(ctx.attrs.link_script, format = "-Clink-arg=-T{}"),
            "-Clink-arg=--no-rosegment",
            "-Cstrip=symbols",
            toolchain.rustc_flags,
            toolchain.rustc_binary_flags,
            ctx.attrs.rustc_flags,
            # The assembly is the whole program only as one fat LTO unit.
            "-Clto=fat",
            "-Ccodegen-units=1",
            cmd_args("--emit=asm=", assembly.as_output(), ",obj=", obj.as_output(), ",link", delimiter = ""),
            "-o",
            image.as_output(),
        ),
        env = ctx.attrs.env | {
            "CARGO_MANIFEST_DIR": srcs,
            # The toolchain's console flags include unstable options.
            "RUSTC_BOOTSTRAP": "1",
            "RUST_TARGET_PATH": toolchain.rust_target_path[DefaultInfo].default_outputs[0],
        },
        category = "rustc",
    )

    return [
        DefaultInfo(default_output = image, sub_targets = {
            "asm": [DefaultInfo(default_output = assembly)],
            "obj": [DefaultInfo(default_output = obj)],
        }),
        Xbox360ExecutableInfo(
            assembly = assembly,
            compiler_builtins = _rlib(compiler_builtins[0]),
            image = image,
            link_script = ctx.attrs.link_script,
            object = obj,
        ),
    ]

_EXECUTABLE_ATTRS = {
    "crate": attrs.string(),
    "crate_root": attrs.string(),
    # Proc macros reach the rule as plugins, built for the execution platform.
    "deps": attrs.list(attrs.dep(providers = [RustLinkInfo], pulls_plugins = [RustProcMacroPlugin]), default = []),
    "edition": attrs.string(),
    "env": attrs.dict(attrs.string(), attrs.string(), default = {}),
    "features": attrs.list(attrs.string(), default = []),
    "labels": attrs.list(attrs.string(), default = []),
    # GNU ld script that places the sections.
    "link_script": attrs.source(),
    # Flags that change the link are rejected; see `_check_rustc_flags`.
    "rustc_flags": attrs.list(attrs.string(), default = []),
    "srcs": attrs.list(attrs.source(), default = []),
    "_rust_lld": attrs.exec_dep(providers = [RunInfo], default = "toolchains//:rust-lld"),
    "_rust_toolchain": toolchains_common.rust(),
}

# A binary crate for the console. The rules configure themselves for the
# console, so host builds include them. The core variant provides core,
# alloc and compiler_builtins; the std variant also std, and the std_system
# variant the std of system modules.
xbox360_executable_core = rule(
    impl = _xbox360_executable_impl,
    attrs = _EXECUTABLE_ATTRS,
    cfg = xbox360_core_transition,
    uses_plugins = [RustProcMacroPlugin],
)

xbox360_executable_std = rule(
    impl = _xbox360_executable_impl,
    attrs = _EXECUTABLE_ATTRS,
    cfg = xbox360_std_transition,
    uses_plugins = [RustProcMacroPlugin],
)

xbox360_executable_std_system = rule(
    impl = _xbox360_executable_impl,
    attrs = _EXECUTABLE_ATTRS,
    cfg = xbox360_std_system_transition,
    uses_plugins = [RustProcMacroPlugin],
)

# Assembles `module.s` with rustc: module-level assembly passes through
# LLVM's assembly parser, as with llvm-mc, for the target specification's
# CPU and features. The crate has no other items, so the object holds only
# the assembly.
_ASSEMBLER_CRATE = """\
#![feature(no_core, rustc_attrs, decl_macro)]
#![allow(internal_features)]
#![no_core]

#[rustc_builtin_macro]
macro global_asm($($arguments:tt)*) {}

#[rustc_builtin_macro]
macro include_str($($arguments:tt)*) {}

global_asm!(include_str!("module.s"), options(raw));
"""

def _assemble(ctx: AnalysisContext, assembly: Artifact) -> Artifact:
    toolchain = ctx.attrs._rust_toolchain[RustToolchainInfo]
    crate = ctx.actions.symlinked_dir("assembler", {
        "module.rs": ctx.actions.write("assembler.rs", _ASSEMBLER_CRATE),
        "module.s": assembly,
    })
    obj = ctx.actions.declare_output("module.o")
    ctx.actions.run(
        cmd_args(
            toolchain.compiler,
            cmd_args(crate, format = "{}/module.rs"),
            "--crate-name=module",
            "--crate-type=lib",
            "--edition=2024",
            "--target=" + toolchain.rustc_target_triple,
            # Custom target specifications are unstable.
            "-Zunstable-options",
            # The CPU and features of the console flags, which the LTO unit
            # was compiled for.
            toolchain.rustc_flags,
            # The crate has no code of its own; the assembly carries the
            # debug information of the LTO unit.
            "-Cdebuginfo=none",
            "--emit=obj",
            # rustc writes its intermediate files next to the output, which
            # is private to this action, rather than in the working directory.
            "-o",
            obj.as_output(),
        ),
        env = {
            "RUSTC_BOOTSTRAP": "1",
            "RUST_TARGET_PATH": toolchain.rust_target_path[DefaultInfo].default_outputs[0],
        },
        category = "assemble",
    )
    return obj

def _link(ctx: AnalysisContext, executable: Xbox360ExecutableInfo, obj: Artifact, name: str, strip: bool) -> Artifact:
    image = ctx.actions.declare_output(name)

    # The command rustc runs to link the executable, with `obj` as its code
    # generation unit. `_check_link` confirms that it is.
    ctx.actions.run(
        cmd_args(
            ctx.attrs._rust_lld[RunInfo],
            "-flavor",
            "gnu",
            obj,
            "--as-needed",
            "-Bstatic",
            executable.compiler_builtins,
            "-Bdynamic",
            "-z",
            "noexecstack",
            "-o",
            image.as_output(),
            "--gc-sections",
            ["--strip-all"] if strip else [],
            cmd_args(executable.link_script, format = "-T{}"),
            "--no-rosegment",
        ),
        category = "link",
        identifier = name,
    )
    return image

def _check_link(ctx: AnalysisContext, executable: Xbox360ExecutableInfo) -> Artifact:
    """Links the object rustc linked as `_link` does, and compares the images."""
    image = _link(ctx, executable, executable.object, "rustc-image", strip = True)
    result = ctx.actions.declare_output("link-check")
    ctx.actions.run(
        cmd_args(
            ctx.attrs._python_bootstrap_toolchain[PythonBootstrapToolchainInfo].interpreter,
            ctx.attrs._compare_files,
            result.as_output(),
            executable.image,
            image,
        ),
        category = "link_check",
    )
    return result

def _xex_from_assembly_impl(ctx: AnalysisContext) -> list[Provider]:
    executable = ctx.attrs.executable[Xbox360ExecutableInfo]
    rewrite = ctx.attrs._asm_rewrite[RunInfo]
    assembly = executable.assembly
    sub_targets = {}
    checks = []

    if ctx.attrs.rewrite:
        rewritten = ctx.actions.declare_output("rewritten.s")
        scope = ctx.actions.declare_output("scope")
        refusals = ctx.actions.declare_output("refusals")
        report = ctx.actions.declare_output("rewrite-report")
        _write_stdout(
            ctx,
            report,
            cmd_args(
                rewrite,
                "rewrite",
                assembly,
                rewritten.as_output(),
                "--scope",
                scope.as_output(),
                "--refusals",
                refusals.as_output(),
            ),
            category = "asm_rewrite",
        )
        assembly = rewritten
        sub_targets["refusals"] = [DefaultInfo(default_output = refusals)]
        sub_targets["rewrite-report"] = [DefaultInfo(default_output = report)]

    obj = _assemble(ctx, assembly)

    # xex-link copies only the loaded sections, so the image with symbols
    # and debug information is also the debugger's ELF.
    image = _link(ctx, executable, obj, "image", strip = False)

    if ctx.attrs.rewrite:
        disassembly = ctx.actions.declare_output("disassembly")
        _write_stdout(
            ctx,
            disassembly,
            cmd_args(ctx.attrs._llvm_objdump[RunInfo], "-d", "--no-show-raw-insn", image),
            category = "disassemble",
        )
        audit = ctx.actions.declare_output("audit")
        _write_stdout(
            ctx,
            audit,
            cmd_args(rewrite, "audit", disassembly, "--scope", scope),
            category = "asm_audit",
        )
        checks.append(audit)
        sub_targets["audit"] = [DefaultInfo(default_output = audit)]
        sub_targets["disassembly"] = [DefaultInfo(default_output = disassembly)]

    reservations = _check_reservations(ctx, image)
    checks.append(reservations)
    checks.append(_check_link(ctx, executable))

    xex = _xex_link(ctx, image, checks)

    sub_targets["asm"] = [DefaultInfo(default_output = assembly)]
    sub_targets["elf"] = [DefaultInfo(default_output = image)]
    sub_targets["reservations"] = [DefaultInfo(default_output = reservations)]
    sub_targets |= _debugger_files(ctx, xex, image, ctx.attrs._sysroot_library, ctx.attrs.rust_project)
    return [
        DefaultInfo(default_output = xex, sub_targets = sub_targets),
        RunInfo(args = _run_title_command(ctx, xex)),
    ]

# A XEX assembled and linked from the assembly of an `xbox360_executable_*`
# target. The rule configures itself for the console; the executable keeps
# its own configuration.
xex_from_assembly = rule(
    impl = _xex_from_assembly_impl,
    cfg = xbox360_core_transition,
    attrs = _XEX_ATTRS | _SYSROOT_LIBRARY_ATTRS | {
        "executable": attrs.dep(providers = [Xbox360ExecutableInfo]),
        "rewrite": attrs.bool(default = True),
        "_compare_files": attrs.source(default = "xbox360//buck:compare_files.py"),
        "_llvm_objdump": attrs.exec_dep(providers = [RunInfo], default = "toolchains//:llvm-objdump"),
        "_rust_lld": attrs.exec_dep(providers = [RunInfo], default = "toolchains//:rust-lld"),
        "_rust_toolchain": toolchains_common.rust(),
    },
)

# The rules of each standard library choice of `xbox360_title`.
_EXECUTABLE_RULES = {
    "core": xbox360_executable_core,
    "std": xbox360_executable_std,
    "std-system": xbox360_executable_std_system,
}

_XEX_RULES = {
    "core": xex_core,
    "std": xex_std,
    "std-system": xex_std_system,
}

def xbox360_title(
        name: str,
        link_script: str | None = None,
        manifest: str | None = None,
        resources: dict[str, str] | str | None = None,
        rust_project: str | None = None,
        std: bool | str = False,
        rewrite: bool = False,
        rustc_flags: list[str] = [],
        visibility: list[str] = [],
        **kwargs):
    """Builds `name.xex` from a Rust executable.

    `manifest` is the xex-link manifest with the title's XEX metadata.
    `link_script` is the GNU ld script that places the sections; by default
    xex-link derives it from the manifest. `resources`
    maps resource names (1 to 8 bytes) to the files, or targets, that the
    image embeds under them, or names an `xex_resources` target.
    `rust_project` names the Cargo manifest used by the `[vscode]` subtarget
    for Rust Analyzer. `std = True` selects the standard library; otherwise
    the crate is `no_std` and may use core and alloc. `std = "system"`
    selects the standard library of system modules: its threads start in the
    system process, which keeps them when the title exits, and its sockets
    open as the system rather than as the title. Other arguments are those
    of `rust_binary`.

    With `rewrite`, the executable is an `xbox360_executable_*` target,
    which takes the `srcs`, `crate_root`, `edition`, `deps`, `features` and
    `env` of `rust_binary`, and `xex_from_assembly` replaces its
    callee-saved register saves and restores with shared helper routines.
    """
    sysroots = {False: "core", True: "std", "system": "std-system"}
    if std not in sysroots:
        fail("{}: std must be False, True or \"system\", not {}".format(name, repr(std)))
    sysroot = sysroots[std]

    crate = kwargs.pop("crate", name.replace("-", "_"))

    # Applies to every rule of the title, so that a title limited to one
    # code generation (such as VMX128 code on PPC64) is skipped elsewhere.
    compatible_with = kwargs.pop("target_compatible_with", [])
    if type(resources) == type({}):
        xex_resources(
            name = name + "-resources",
            files = resources,
        )
        resources = ":{}-resources".format(name)
    if link_script:
        native.export_file(
            name = name + "-link-script",
            src = link_script,
        )
    else:
        xex_link_script(
            name = name + "-link-script",
            manifest = manifest,
        )
    if rewrite:
        _EXECUTABLE_RULES[sysroot](
            name = name + "-executable",
            crate = crate,
            link_script = ":{}-link-script".format(name),
            rustc_flags = rustc_flags,
            target_compatible_with = compatible_with,
            **kwargs
        )
        xex_from_assembly(
            name = name,
            executable = ":{}-executable".format(name),
            manifest = manifest,
            resources = resources,
            rust_project = rust_project,
            target_compatible_with = compatible_with,
            visibility = visibility,
        )
        return

    native.rust_binary(
        name = name + "-elf",
        crate = crate,
        # A static link strategy keeps the target's static relocation model;
        # the prelude otherwise compiles position-independent code.
        link_style = "static",
        rustc_flags = [
            "-Clink-arg=-T$(location :{}-link-script)".format(name),
            "-Clink-arg=--no-rosegment",
        ] + rustc_flags,
        target_compatible_with = ["xbox360//platforms:target[xbox]"] + compatible_with,
        **kwargs
    )
    _XEX_RULES[sysroot](
        name = name,
        elf = ":{}-elf".format(name),
        manifest = manifest,
        rust_project = rust_project,
        resources = resources,
        target_compatible_with = compatible_with,
        visibility = visibility,
    )
