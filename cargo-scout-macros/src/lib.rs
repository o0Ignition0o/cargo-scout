#[macro_export(local_inner_macros)]
macro_rules! success {
    ($value:expr) => {
        std::println!("{}", $value.green());
    };
    ($format:expr, $($arg:tt)+) => {
        std::println!("{}", std::format!($format, $($arg)+).green());
    };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($value:expr) => {
        std::println!("{}", $value.cyan());
    };
    ($format:expr, $($arg:tt)+) => {
        std::println!("{}", std::format!($format, $($arg)+).cyan());
    };
}

#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($value:expr) => {
        std::println!("{}", $value.yellow());
    };
    ($format:expr, $($arg:tt)+) => {
        std::println!("{}", std::format!($format, $($arg)+).yellow());
    };
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($value:expr) => {
        std::println!("{}", $value.red());
    };
    ($format:expr, $($arg:tt)+) => {
        std::println!("{}", std::format!($format, $($arg)+).red());
    };
}
