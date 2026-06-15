<a name="top"></a>
<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=0:E50914,100:0B0B0B&height=200&section=header&text=eos-kernel&fontSize=68&fontColor=ffffff&fontAlignY=38&desc=The%20E-OS%20microkernel%20%28Rust%29&descAlignY=60&descSize=18&animation=fadeIn" alt="eos-kernel"/>
</p>

<p align="center">
  <img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=19&pause=900&color=E50914&center=true&vCenter=true&width=820&lines=The+E-OS+microkernel%2C+written+in+Rust;Downstream+of+redox-os%2Fkernel;aarch64+FEAT_RNG+%2B+PCIe+INTx+fixes" alt="tagline"/>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-E50914?style=for-the-badge&labelColor=0B0B0B" alt="license"/>
  <img src="https://img.shields.io/badge/Rust-E50914?style=for-the-badge&logo=rust&logoColor=white&labelColor=0B0B0B" alt="Rust"/>
  <img src="https://img.shields.io/badge/kernel-microkernel-E50914?style=for-the-badge&labelColor=0B0B0B" alt="microkernel"/>
  <img src="https://img.shields.io/badge/part%20of-E--OS-E50914?style=for-the-badge&labelColor=0B0B0B" alt="part of E-OS"/>
</p>

<p align="center"><img src="https://raw.githubusercontent.com/Gh0s777tt/Gh0s777tt/main/assets/divider.svg" width="100%" alt=""/></p>

## Requirements

* [`nasm`](https://nasm.us/) needs to be available on the PATH at build time.

## Building The Documentation

Use this command:

```sh
cargo doc --open --target x86_64-unknown-none
```

## Debugging

### QEMU

Running [QEMU](https://www.qemu.org) with the `-s` flag will set up QEMU to listen on port `1234` for a GDB client to connect to it. To debug the redox kernel run.

```sh
make qemu gdb=yes
```

This will start a virtual machine with and listen on port `1234` for a GDB or LLDB client.

### GDB

If you are going to use [GDB](https://www.gnu.org/software/gdb/), run these commands to load debug symbols and connect to your running kernel:

```
(gdb) symbol-file build/kernel.sym
(gdb) target remote localhost:1234
```

### LLDB

If you are going to use [LLDB](https://lldb.llvm.org/), run these commands to start debugging:

```
(lldb) target create -s build/kernel.sym build/kernel
(lldb) gdb-remote localhost:1234
```

After connecting to your kernel you can set some interesting breakpoints and `continue`
the process. See your debuggers man page for more information on useful commands to run.

## Notes

- Always use `foo.get(n)` instead of `foo[n]` and try to cover for the possibility of `Option::None`. Doing the regular way may work fine for applications, but never in the kernel. No possible panics should ever exist in kernel space, because then the whole OS would just stop working.

- If you receive a kernel panic in QEMU, use `pkill qemu-system` to kill the frozen QEMU process.

## How To Contribute

To learn how to contribute to this system component you need to read the following document:

- [CONTRIBUTING.md](https://gitlab.redox-os.org/redox-os/redox/-/blob/master/CONTRIBUTING.md)

## Development

To learn how to do development with this system component inside the Redox build system you need to read the [Build System](https://doc.redox-os.org/book/build-system-reference.html) and [Coding and Building](https://doc.redox-os.org/book/coding-and-building.html) pages.

### How To Build

To build this system component you need to download the Redox build system, you can learn how to do it on the [Building Redox](https://doc.redox-os.org/book/podman-build.html) page.

This is necessary because they only work with cross-compilation to a Redox virtual machine, but you can do some testing from Linux.

## Funding - _Unix-style Signals and Process Management_

This project is funded through [NGI Zero Core](https://nlnet.nl/core), a fund established by [NLnet](https://nlnet.nl) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu) program. Learn more at the [NLnet project page](https://nlnet.nl/project/RedoxOS-Signals).

[<img src="https://nlnet.nl/logo/banner.png" alt="NLnet foundation logo" width="20%" />](https://nlnet.nl)
[<img src="https://nlnet.nl/image/logos/NGI0_tag.svg" alt="NGI Zero Logo" width="20%" />](https://nlnet.nl/core)

<p align="center"><img src="https://raw.githubusercontent.com/Gh0s777tt/Gh0s777tt/main/assets/divider.svg" width="100%" alt=""/></p>

<div align="center">

### 🩸 Part of the **GHOST EMPIRE** ecosystem

[**E-OS**](https://github.com/Gh0s777tt/E-OS) · Rust microkernel OS &nbsp;·&nbsp; Minecraft infrastructure suite &nbsp;·&nbsp; Discord &amp; streaming platforms — forged under **Empire Forge**.

<a href="https://discord.gg/Egf88V9UdH"><img src="https://img.shields.io/badge/Discord-Join%20the%20Empire-5865F2?style=for-the-badge&logo=discord&logoColor=white&labelColor=0B0B0B" alt="discord"/></a>
<a href="mailto:ghostt77@empire-forge.com"><img src="https://img.shields.io/badge/Email-Empire%20Forge-E50914?style=for-the-badge&logo=maildotru&logoColor=white&labelColor=0B0B0B" alt="email"/></a>
<a href="https://donatr.ee/ghost77/"><img src="https://img.shields.io/badge/%E2%9D%A4%20Support-Donate-E50914?style=for-the-badge&labelColor=0B0B0B" alt="donate"/></a>

<sub><i>Black. Red. Production-grade. — © GHOST EMPIRE · Empire Forge</i></sub>

</div>
