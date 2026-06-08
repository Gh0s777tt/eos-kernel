use ::syscall::Exception;
use rmm::VirtualAddress;

use crate::{
    context::signal::excp_handler,
    exception_stack,
    memory::{ArchIntCtx, GenericPfFlags},
    sync::CleanLockToken,
    syscall,
};

use super::InterruptStack;

exception_stack!(synchronous_exception_at_el1_with_sp0, |stack| {
    println!("Synchronous exception at EL1 with SP0");
    stack.trace();
    loop {}
});

fn exception_code(esr: usize) -> u8 {
    ((esr >> 26) & 0x3f) as u8
}
fn iss(esr: usize) -> u32 {
    (esr & 0x01ff_ffff) as u32
}

unsafe fn far_el1() -> usize {
    unsafe {
        let ret: usize;
        core::arch::asm!("mrs {}, far_el1", out(reg) ret);
        ret
    }
}

unsafe fn instr_data_abort_inner(
    stack: &mut InterruptStack,
    from_user: bool,
    instr_not_data: bool,
    _from: &str,
) -> bool {
    unsafe {
        let iss = iss(stack.iret.esr_el1);
        let fsc = iss & 0x3F;
        //dbg!(fsc);

        let was_translation_fault = fsc >= 0b000100 && fsc <= 0b000111;
        //let was_permission_fault = fsc >= 0b001101 && fsc <= 0b001111;
        let write_not_read_if_data = iss & (1 << 6) != 0;

        let mut flags = GenericPfFlags::empty();
        flags.set(GenericPfFlags::PRESENT, !was_translation_fault);

        // TODO: RMW instructions may "involve" writing to (possibly invalid) memory, but AArch64
        // doesn't appear to require that flag to be set if the read alone would trigger a fault.
        flags.set(
            GenericPfFlags::INVOLVED_WRITE,
            write_not_read_if_data && !instr_not_data,
        );
        flags.set(GenericPfFlags::INSTR_NOT_DATA, instr_not_data);
        flags.set(GenericPfFlags::USER_NOT_SUPERVISOR, from_user);

        let faulting_addr = VirtualAddress::new(far_el1());
        //dbg!(faulting_addr, flags, from);

        crate::memory::page_fault_handler(stack, flags, faulting_addr).is_ok()
    }
}

unsafe fn cntfrq_el0() -> usize {
    unsafe {
        let ret: usize;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) ret);
        ret
    }
}

unsafe fn cntpct_el0() -> usize {
    unsafe {
        let ret: usize;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) ret);
        ret
    }
}

unsafe fn cntvct_el0() -> usize {
    unsafe {
        let ret: usize;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) ret);
        ret
    }
}

unsafe fn instr_trapped_msr_mrs_inner(
    stack: &mut InterruptStack,
    _from_user: bool,
    _instr_not_data: bool,
    _from: &str,
) -> bool {
    unsafe {
        let iss = iss(stack.iret.esr_el1);
        // let res0 = (iss & 0x1C0_0000) >> 22;
        let op0 = (iss & 0x030_0000) >> 20;
        let op2 = (iss & 0x00e_0000) >> 17;
        let op1 = (iss & 0x001_c000) >> 14;
        let crn = (iss & 0x000_3c00) >> 10;
        let rt = (iss & 0x000_03e0) >> 5;
        let crm = (iss & 0x000_001e) >> 1;
        let dir = iss & 0x000_0001;

        /*
        print!("iss=0x{:x}, res0=0b{:03b}, op0=0b{:02b}\n
                op2=0b{:03b}, op1=0b{:03b}, crn=0b{:04b}\n
                rt=0b{:05b}, crm=0b{:04b}, dir=0b{:b}\n",
                iss, res0, op0, op2, op1, crn, rt, crm, dir);
        */

        match (op0, op1, crn, crm, op2, dir) {
            //MRS <Xt>, CNTFRQ_EL0
            (0b11, 0b011, 0b1110, 0b0000, 0b000, 0b1) => {
                let reg_val = cntfrq_el0();
                stack.store_reg(rt as usize, reg_val);
                //skip faulting instruction, A64 instructions are always 32-bits
                stack.iret.elr_el1 += 4;
                return true;
            }
            //MRS <Xt>, CNTPCT_EL0
            (0b11, 0b011, 0b1110, 0b0000, 0b001, 0b1) => {
                let reg_val = cntpct_el0();
                stack.store_reg(rt as usize, reg_val);
                //skip faulting instruction, A64 instructions are always 32-bits
                stack.iret.elr_el1 += 4;
                return true;
            }
            //MRS <Xt>, CNTVCT_EL0
            (0b11, 0b011, 0b1110, 0b0000, 0b010, 0b1) => {
                let reg_val = cntvct_el0();
                stack.store_reg(rt as usize, reg_val);
                //skip faulting instruction, A64 instructions are always 32-bits
                stack.iret.elr_el1 += 4;
                return true;
            }
            _ => {}
        }

        false
    }
}

exception_stack!(synchronous_exception_at_el1_with_spx, |stack| {
    unsafe {
        if !pf_inner(
            stack,
            exception_code(stack.iret.esr_el1),
            "sync_exc_el1_spx",
        ) {
            println!("Synchronous exception at EL1 with SPx");
            if exception_code(stack.iret.esr_el1) == 0b100101 {
                let far_el1 = far_el1();
                println!("FAR_EL1 = 0x{:08x}", far_el1);
            } else if exception_code(stack.iret.esr_el1) == 0b100100 {
                let far_el1 = far_el1();
                println!("USER FAR_EL1 = 0x{:08x}", far_el1);
            }
            stack.trace();
            loop {}
        }
    }
});
unsafe fn pf_inner(stack: &mut InterruptStack, ty: u8, from: &str) -> bool {
    unsafe {
        match ty {
            // "Data Abort taken from a lower Exception level"
            0b100100 => instr_data_abort_inner(stack, true, false, from),
            // "Data Abort taken without a change in Exception level"
            0b100101 => instr_data_abort_inner(stack, false, false, from),
            // "Instruction Abort taken from a lower Exception level"
            0b100000 => instr_data_abort_inner(stack, true, true, from),
            // "Instruction Abort taken without a change in Exception level"
            0b100001 => instr_data_abort_inner(stack, false, true, from),
            // "Trapped MSR, MRS or System instruction execution in AArch64 state"
            0b011000 => instr_trapped_msr_mrs_inner(stack, true, true, from),

            _ => return false,
        }
    }
}

/// E-OS R-401b: software emulation of the FEAT_RNG (ARMv8.5) RNDR / RNDRRS system registers for
/// CPUs that do not implement them (e.g. ARMv8.0 cortex-a72, Raspberry Pi 3/4). Redox `randd`
/// reads RNDRRS unconditionally to seed the system CSPRNG; on a non-FEAT_RNG core this is an
/// UNDEFINED instruction -> synchronous exception with EC=0b000000 ("Unknown reason") at EL0,
/// which kills randd, takes down the `rand:` scheme, and cascades into a failed boot (every later
/// daemon panics with "failed to generate random data: ENODEV").
///
/// Returns true iff the faulting instruction was `MRS <Xt>, RNDR/RNDRRS` and was emulated.
///
/// ENTROPY (E-OS R-401b, enhanced): rather than a single CNTPCT seed feeding a deterministic
/// splitmix64, every read folds in fresh **CPU-execution-timing jitter** -- the low bits of
/// CNTVCT_EL0 deltas measured across short bursts of data-dependent memory work. On real
/// non-FEAT_RNG hardware (cortex-a72 etc.) those deltas vary unpredictably from cache/pipeline/DVFS
/// noise, giving a genuine (if modest) entropy source -- the CPU-jitter technique used by Linux's
/// jitterentropy / haveged. A Weyl counter + splitmix64 finalizer form the backbone so the output
/// is always non-repeating and non-zero even where the jitter is weak (e.g. a deterministic
/// emulator like QEMU TCG). This is still not a certified TRNG -- a real hardware RNG remains the
/// ideal -- but it is materially stronger than the old single-seed PRNG.
unsafe fn emulate_feat_rng(stack: &mut InterruptStack) -> bool {
    use core::sync::atomic::{AtomicU64, Ordering};
    // Persistent entropy pool, stirred every call. WEYL is a per-call monotonic counter (atomic
    // RMW) that guarantees distinct, non-repeating output even across CPUs and even if the pool
    // and jitter were to contribute nothing on a given platform.
    static POOL: AtomicU64 = AtomicU64::new(0);
    static WEYL: AtomicU64 = AtomicU64::new(0);

    unsafe {
        // Read the faulting instruction from EL0 via an unprivileged load (respects PAN).
        let elr = stack.iret.elr_el1;
        let instr: u32;
        core::arch::asm!(
            "ldtr {0:w}, [{1}]",
            out(reg) instr,
            in(reg) elr,
            options(nostack, readonly, preserves_flags),
        );

        // MRS <Xt>, RNDR   = 0xD53B_2400 | Rt
        // MRS <Xt>, RNDRRS = 0xD53B_2420 | Rt
        let masked = instr & 0xFFFF_FFE0;
        if masked != 0xD53B_2400 && masked != 0xD53B_2420 {
            return false;
        }
        let rt = (instr & 0x1F) as usize;

        // --- gather CPU-execution-timing jitter (the real entropy source on hardware) ---
        // Sample CNTVCT_EL0 around short, data-dependent, variable-latency memory work; the timing
        // deltas carry microarchitectural noise. `black_box` defeats the optimizer so the work and
        // the loop actually execute (and are not folded into a constant).
        let mut jitter: u64 = cntvct_el0() as u64;
        let mut scratch: [u64; 8] = [
            0x243F_6A88_85A3_08D3, 0x1319_8A2E_0370_7344,
            0xA409_3822_299F_31D0, 0x082E_FA98_EC4E_6C89,
            0x4528_21E6_38D0_1377, 0xBE54_66CF_34E9_0C6C,
            0xC0AC_29B7_C97C_50DD, 0x3F84_D5B5_B547_0917,
        ];
        for round in 0..48u64 {
            let t0 = cntvct_el0() as u64;
            let idx = ((jitter ^ round) & 7) as usize;
            scratch[idx] = scratch[idx]
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .rotate_left((t0 & 63) as u32)
                ^ jitter;
            core::hint::black_box(&scratch);
            let t1 = cntvct_el0() as u64;
            jitter = jitter
                .rotate_left(7)
                ^ t1.wrapping_sub(t0)
                ^ scratch[idx]
                ^ t1;
        }

        // --- stir the persistent pool with the gathered jitter + both timers ---
        let weyl = WEYL
            .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut pool = POOL.load(Ordering::Relaxed);
        if pool == 0 {
            pool = (cntpct_el0() as u64) ^ 0x9E37_79B9_7F4A_7C15;
        }
        pool = pool
            .rotate_left(17)
            .wrapping_add(jitter)
            ^ (cntpct_el0() as u64)
            ^ weyl;
        POOL.store(pool, Ordering::Relaxed);

        // --- splitmix64 finalizer over (pool + weyl) for a well-distributed output ---
        let mut z = pool.wrapping_add(weyl);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // randd treats a 0 result as "RNG not ready"; never hand back exactly 0.
        if z == 0 {
            z = 0x9E37_79B9_7F4A_7C15;
        }

        // store_reg() treats rt==31 (XZR) as a no-op, matching the CNTxxx emulation above.
        stack.store_reg(rt, z as usize);
        // FEAT_RNG sets PSTATE.NZCV = 0b0000 on success; NZCV = SPSR_EL1[31:28].
        stack.iret.spsr_el1 &= !(0xF << 28);
        // Skip the emulated instruction (A64 instructions are always 4 bytes).
        stack.iret.elr_el1 += 4;
        true
    }
}

exception_stack!(synchronous_exception_at_el0, |stack| {
    unsafe {
        match exception_code(stack.iret.esr_el1) {
            0b010101 => {
                let scratch = &stack.scratch;
                let mut token = CleanLockToken::new();
                let ret = syscall::syscall(
                    scratch.x8, scratch.x0, scratch.x1, scratch.x2, scratch.x3, scratch.x4,
                    scratch.x5, &mut token,
                );
                stack.scratch.x0 = ret;
            }

            ty => {
                if !pf_inner(stack, ty as u8, "sync_exc_el0")
                    // E-OS R-401b: emulate FEAT_RNG (RNDR/RNDRRS) on cores that lack it, instead of
                    // killing the process. EC=0b000000 is "Unknown reason" (undefined instruction).
                    && !(ty == 0b000000 && emulate_feat_rng(stack))
                {
                    error!(
                        "FATAL: Not an SVC induced synchronous exception (ty={:b})",
                        ty
                    );
                    println!("FAR_EL1: {:#0x}", far_el1());
                    //crate::debugger::debugger(None);
                    stack.trace();
                    excp_handler(Exception {
                        kind: 0, // TODO
                    });
                }
            }
        }
    }
});

exception_stack!(unhandled_exception, |stack| {
    println!("Unhandled exception");
    stack.trace();
    loop {}
});

impl ArchIntCtx for InterruptStack {
    fn ip(&self) -> usize {
        self.iret.elr_el1
    }
    fn recover_and_efault(&mut self) {
        // Set the return value to nonzero to indicate usercopy failure (EFAULT), and emulate the
        // return instruction by setting the return pointer to the saved LR value.

        self.iret.elr_el1 = self.preserved.x30;
        self.scratch.x0 = 1;
    }
}
