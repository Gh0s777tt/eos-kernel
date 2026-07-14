# eos-kernel

**E-OS fork of [`redox-os/kernel`](https://gitlab.redox-os.org/redox-os/kernel).** Part of the [**E-OS**](https://github.com/Gh0s777tt/E-OS) ecosystem — a hardened, Crimson-branded downstream of [Redox OS](https://www.redox-os.org).

This repository is the **Redox microkernel**.

## E-OS changes vs upstream

- **Memory-safety hardening upstream lacks** — userspace mmap **ASLR**, **W^X** enforcement, guard bands around map-anywhere, release overflow-checks (fail-secure).
- **aarch64 platform (R-401 / R-502a)** — in-kernel INTx mask+EOI (fixes a virtio deadlock), shared-phandle IRQ opens for PCIe INTx, **FEAT_RNG (RNDR/RNDRRS) emulation**, virtual-timer IRQ, PL011 serial RXE init, cumulative-ISAR AES/PMULL/SHA2 detection.

## How it's pinned

The E-OS build pins this fork in [`recipes/core/kernel/recipe.toml`](https://github.com/Gh0s777tt/E-OS/blob/main/recipes/core/kernel/recipe.toml):

- branch **`eos-july`** · rev **`c918080f13fc`**
- **4 commit(s) behind** upstream master

## Build standalone

This fork is normally built by the E-OS cookbook (`make CI=1 …` in the [main repo](https://github.com/Gh0s777tt/E-OS)). To build it on its own you need the Redox toolchain; see the main repo's [build guide](https://github.com/Gh0s777tt/E-OS/blob/main/docs/building.md).

## Hosting

**GitLab (source of truth):** https://gitlab.com/e-os/eos-kernel  
**GitHub (read-only mirror):** https://github.com/Gh0s777tt/eos-kernel

## License

MIT (inherited from upstream Redox). The E-OS project as a whole is AGPL-3.0; see the [main repo](https://github.com/Gh0s777tt/E-OS/blob/main/LICENSE).

---
[E-OS main repo](https://github.com/Gh0s777tt/E-OS) · [Docs](https://github.com/Gh0s777tt/E-OS/tree/main/docs) · [Upstream](https://gitlab.redox-os.org/redox-os/kernel)
