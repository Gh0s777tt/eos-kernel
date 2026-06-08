use crate::{
    context,
    sync::CleanLockToken,
    syscall::{
        data::TimeSpec,
        error::*,
        flag::{CLOCK_MONOTONIC, CLOCK_REALTIME},
    },
    time,
};

use super::usercopy::{UserSliceRo, UserSliceWo};

pub fn clock_gettime(clock: usize, buf: UserSliceWo, token: &mut CleanLockToken) -> Result<()> {
    let arch_time = match clock {
        CLOCK_REALTIME => time::realtime(token),
        CLOCK_MONOTONIC => time::monotonic(token),
        _ => return Err(Error::new(EINVAL)),
    };

    buf.copy_exactly(&TimeSpec::from_nanos(arch_time))
}

/// Nanosleep will sleep by switching the current context
pub fn nanosleep(
    req_buf: UserSliceRo,
    rem_buf_opt: Option<UserSliceWo>,
    token: &mut CleanLockToken,
) -> Result<()> {
    let req = unsafe { req_buf.read_exact::<TimeSpec>()? };

    if req.tv_sec < 0 || req.tv_nsec < 0 || req.tv_nsec >= time::NANOS_PER_SEC as i32 {
        return Err(Error::new(EINVAL));
    }

    let start = time::monotonic(token);
    let end = start + req.to_nanos();

    let current_context = context::current();
    {
        let context = current_context.upgradeable_read(token.token());

        if let Some((tctl, pctl, _)) = context.sigcontrol()
            && tctl.currently_pending_unblocked(pctl) != 0
        {
            return Err(Error::new(EINTR));
        }
        let mut context = context.upgrade();
        context.wake = Some(end);
        context.block("nanosleep");
    }

    // TODO: The previous wakeup reason was most likely signals, but is there any other possible
    // reason?
    context::switch(token);

    let was_interrupted = current_context.write(token.token()).wake.take().is_some();

    if let Some(rem_buf) = rem_buf_opt {
        let current = time::monotonic(token);

        rem_buf.copy_exactly(&if current < end {
            let diff = end - current;
            TimeSpec::from_nanos(diff)
        } else {
            TimeSpec::default()
        })?;
    }

    if was_interrupted {
        Err(Error::new(EINTR))
    } else {
        Ok(())
    }
}

pub fn sched_yield(token: &mut CleanLockToken) -> Result<()> {
    context::switch(token);

    // E-OS R-401e: aarch64 syscall-return vs signal-delivery ordering fix.
    //
    // On aarch64, `InterruptStack::sig_archdep_reg()` returns `scratch.x0` -- but x0 is ALSO
    // the syscall return register. `signal_handler()` (below) saves `sig_archdep_reg()` into
    // `saved_archdep_reg`, redirects the frame to the signal trampoline, and relibc's sigreturn
    // later restores x0 from that saved value. However, the aarch64 SVC handler commits the
    // syscall result (`stack.scratch.x0 = ret`) only AFTER `syscall()` returns -- i.e. after
    // this function. So if a signal is delivered to a context during its yield, the value saved
    // here is the stale syscall *input* (e.g. relibc verify()'s YIELD passes !0 in every arg),
    // and sigreturn clobbers the real return (0) with it. The interrupted code then sees x0=-1,
    // which deterministically breaks the first signal-receiving fork+exec'd program on aarch64
    // (manifested as relibc verify() aborting every shell/desktop process).
    //
    // The other arches are immune: x86/x86_64 use the flags register and riscv64 uses a
    // temporary (t0) as the archdep reg -- none of which carry the syscall return value.
    //
    // Fix: commit the yield's return value (always Ok(()) == 0) into the frame before the
    // signal check, so `sig_archdep_reg()` reflects the completed syscall. Harmless even when
    // no signal is pending (the SVC handler writes the same 0 afterwards). Scoped to aarch64,
    // where x0 is the return register; the guards are dropped before `signal_handler` re-locks.
    #[cfg(target_arch = "aarch64")]
    {
        let context_lock = context::current();
        let mut context = context_lock.upgradeable_read(token.token()).upgrade();
        if let Some(regs) = context.regs_mut() {
            regs.scratch.x0 = 0;
        }
    }

    // TODO: Do this check in userspace
    context::signal::signal_handler(token);
    Ok(())
}
