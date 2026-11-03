# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""`xeedee` and `openxbdm-kd` from an openxbdm release archive that Buck2
downloads and checks by SHA-256.

The archive is selected by the os and cpu of the execution platform, so reach
the tools through `attrs.exec_dep`. A host that the release has no archive for
gets targets without RunInfo; `xbox360.xbdm_tool` and `xbox360.kd_tool` name
the tools there.
"""

load("@prelude//http_archive:http_archive.bzl", "http_archive_impl")
load("@prelude//rules.bzl", "clone_rule")

_RELEASE = "nightly-57c60d10a769"
_RELEASE_URL = "https://github.com/landaire/openxbdm/releases/download/{}/".format(_RELEASE)

# os -> cpu -> the release archive and the directory it holds.
_HOSTS = {
    "linux": {
        "x86_64": struct(
            archive = "xeedee-nightly-x86_64-unknown-linux-gnu.tar.gz",
            sha256 = "4c033ec015fa5c48bcb1384ee88f32fea559c3b0beca4f577f2647f68853e8a7",
            strip_prefix = "xeedee-nightly-x86_64-unknown-linux-gnu",
            type = "tar.gz",
        ),
    },
    "macos": {
        "arm64": struct(
            archive = "xeedee-nightly-aarch64-apple-darwin.tar.gz",
            sha256 = "553500680eb736138eba04127e9bdb65552d1a2ba6f13411bb0f5204863fe30d",
            strip_prefix = "xeedee-nightly-aarch64-apple-darwin",
            type = "tar.gz",
        ),
    },
    "windows": {
        "x86_64": struct(
            archive = "xeedee-nightly-x86_64-pc-windows-msvc.zip",
            sha256 = "a1607fb19f661e66b0f6ed1a054fba3714c65ec176652de75ba76d47b5466fcf",
            strip_prefix = "xeedee-nightly-x86_64-pc-windows-msvc",
            type = "zip",
        ),
    },
}

def _archive_impl(ctx: AnalysisContext) -> list[Provider]:
    # The prelude's http_archive also returns a test that only materializes the archive.
    return [p for p in http_archive_impl(ctx) if not isinstance(p, ExternalRunnerTestInfo)]

_archive = clone_rule("http_archive", impl_override = _archive_impl)

def _archive_tool_impl(ctx: AnalysisContext) -> list[Provider]:
    if ctx.attrs.archive == None:
        return [DefaultInfo()]
    directory = ctx.attrs.archive[DefaultInfo].default_outputs[0]
    return [
        DefaultInfo(default_output = directory),
        RunInfo(args = cmd_args(directory, format = "{}/" + ctx.attrs.executable)),
    ]

_archive_tool = rule(
    impl = _archive_tool_impl,
    attrs = {
        "archive": attrs.option(attrs.dep(), default = None),
        "executable": attrs.string(),
    },
)

def openxbdm_host_tools():
    for os_name, cpus in _HOSTS.items():
        for cpu, release in cpus.items():
            _archive(
                name = "openxbdm-tools-{}-{}".format(os_name, cpu),
                sha256 = release.sha256,
                strip_prefix = release.strip_prefix,
                type = release.type,
                urls = [_RELEASE_URL + release.archive],
            )

    archive = select({"DEFAULT": None} | {
        "prelude//os:" + os_name: select({"DEFAULT": None} | {
            "prelude//cpu:" + cpu: ":openxbdm-tools-{}-{}".format(os_name, cpu)
            for cpu in cpus
        })
        for os_name, cpus in _HOSTS.items()
    })
    for tool in ["xeedee", "openxbdm-kd"]:
        _archive_tool(
            name = tool,
            archive = archive,
            executable = select({
                "DEFAULT": tool,
                "prelude//os:windows": tool + ".exe",
            }),
            visibility = ["PUBLIC"],
        )
