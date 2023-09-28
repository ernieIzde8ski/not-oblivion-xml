#[macro_export]
macro_rules! debug {
    () => {
        eprintln!("Ln: {}", line!());
    };
    ($($arg:tt)*) => {{
        eprint!("Ln: {}\t|\t", line!());
        eprintln!($($arg)*);
    }};
}
