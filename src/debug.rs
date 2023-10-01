#[macro_export]
macro_rules! debug {
    () => {
        eprintln!("{:>25}:{:03}", file!(), line!());
    };
    ($($arg:tt)*) => {{
        eprint!("{:>25}:{:03}  |  ", file!(), line!());
        eprintln!($($arg)*);
    }};
}
