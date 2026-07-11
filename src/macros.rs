/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::log::Writer::new(), $($arg)*);
    });
}

/// Print with new line to console
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = writeln!($crate::log::Writer::new(), $($arg)*);
    });
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        println!("{}:ERROR -- {}", core::module_path!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("{}:WARN -- {}", core::module_path!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("{}:INFO -- {}", core::module_path!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        // E-OS: gate the aarch64/riscv64 debug output behind KERNEL_DEBUG (default
        // false). Upstream printed it unconditionally on those arches -- a bring-up
        // aid that floods the console (one DEBUG per call_fdread etc.) and starves
        // interactive serial input under QEMU TCG. Set KERNEL_DEBUG=true to re-enable.
        if cfg!(any(target_arch = "aarch64", target_arch = "riscv64")) && $crate::KERNEL_DEBUG {
            println!("{}:DEBUG -- {}", core::module_path!(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if false {
            println!("{}:TRACE -- {}", core::module_path!(), format_args!($($arg)*));
        }
    };
}
