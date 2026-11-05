"""Generated pin for the latest tested prebuilt PPC64 toolchain."""

DEFAULT_PPC64_TOOLCHAIN = "prebuilt"

NIGHTLY_TOOLCHAIN = {
    "hosts": {
        "linux": {
            "x86_64": {
                "sha256": "42d260fa11bac40e99d190ebe1448d38ff6bac150293e72517a84a6ff237dcce",
                "strip_prefix": "xbox360-toolchain-x86_64-unknown-linux-gnu",
                "url": "https://github.com/landaire/xenon-forge/releases/download/toolchain-nightly-17e188e5bd9f/xbox360-toolchain-x86_64-unknown-linux-gnu.tar.xz",
            },
        },
        "macos": {
            "arm64": {
                "sha256": "a03bea9053144538ce5d09a0ac3ea9e02185618bf5b6d39c44e6b13510637ec2",
                "strip_prefix": "xbox360-toolchain-aarch64-apple-darwin",
                "url": "https://github.com/landaire/xenon-forge/releases/download/toolchain-nightly-17e188e5bd9f/xbox360-toolchain-aarch64-apple-darwin.tar.xz",
            },
        },
        "windows": {
            "x86_64": {
                "sha256": "7454011b1de19624865f8601774baa9d82cb7b384a1ac4a428a0ee788bbb7ebf",
                "strip_prefix": "xbox360-toolchain-x86_64-pc-windows-msvc",
                "url": "https://github.com/landaire/xenon-forge/releases/download/toolchain-nightly-17e188e5bd9f/xbox360-toolchain-x86_64-pc-windows-msvc.tar.gz",
            },
        },
    },
    "source_commit": "f369053666c0d56176c8fdefd7d87f3dcc168c63",
}
