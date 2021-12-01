#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        #[cfg(any(feature = "trace", feature = "debug", feature = "info", feature = "warn"))]
        println!(
            "[\x1b[{}mWARN \x1b[0m {}] {}",
            $crate::logger::level2color($crate::logger::Level::Warn),
            $crate::arch::cpu::get_cpu_id(),
            format_args!($($arg)*)
        );
    })
}

