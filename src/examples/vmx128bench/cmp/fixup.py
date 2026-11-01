#!/usr/bin/env python3
"""Prepares compiler assembly for inclusion with global_asm!.

    fixup.py PREFIX IN.s OUT.s [HEADER-LINE...]

- Drops .file/.ident/.addrsig and the GNU-stack note.
- Prefixes local labels (.Lxxx -> .L<PREFIX>_xxx): every included file
  shares one assembler scope.
- Calls core's bounds-check panic by a stable name. The mangled name
  carries the hash of the core crate, which changes with the build
  configuration; the title defines the stable one.
"""

import re
import sys

PANIC_BOUNDS_CHECK = re.compile(r'_RNvNtCs[A-Za-z0-9_]+_4core9panicking18panic_bounds_check')

def main():
    prefix, src, dst = sys.argv[1:4]
    header = sys.argv[4:]
    out = []
    for line in open(src):
        if re.match(r'^\s*\.(file|ident|addrsig)\b', line) or '.note.GNU-stack' in line:
            continue
        line = re.sub(r'\.L([A-Za-z_])', r'.L%s_\1' % prefix, line)
        line = PANIC_BOUNDS_CHECK.sub('__vmx128bench_panic_bounds_check', line)
        out.append(line)
    with open(dst, "w") as f:
        for h in header:
            f.write("# %s\n" % h)
        f.writelines(out)


if __name__ == "__main__":
    main()
