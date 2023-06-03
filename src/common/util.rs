

#[macro_export]
macro_rules! noop {
    ($($tt:tt)*) => {};
}

#[macro_export]
macro_rules! black_box {
    ($expr:expr) => {
        ::std::hint::black_box($expr);
    };
}

#[cfg(test)]
pub fn setup_test() {
    use std::sync::Once;
    use tracing::Level;
    static LOGGING: Once = Once::new();
    LOGGING.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();
    })
}

#[cfg(test)]
pub fn mock_interpret(source: &str) -> String {
    let mut stderr = vec![];
    let mut stdout = vec![];
    let _ = crate::interpret(source, &mut stderr, &mut stdout);
    let stderr = String::from_utf8(strip_ansi_escapes::strip(stderr).unwrap()).unwrap();
    let stdout = String::from_utf8(strip_ansi_escapes::strip(stdout).unwrap()).unwrap();
    format!("stdout:\n{stdout}\n\nstderr:\n{stderr}\n")
}

#[cfg(test)]
pub fn mock_parse(source: &str) -> String {
    let mut stderr = vec![];
    let ast = crate::compiler::parse(source, &mut stderr).unwrap();
    let stderr = String::from_utf8(strip_ansi_escapes::strip(stderr).unwrap()).unwrap();
    format!("AST:\n{ast}\n\nstderr:\n{stderr}\n")
}

#[cfg(test)]
#[cfg(feature = "snap")]
pub use ::insta::assert_snapshot;
#[cfg(test)]
#[cfg(not(feature = "snap"))]
pub use black_box as assert_snapshot;

#[cfg(test)]
#[macro_export]
macro_rules! snap {
    ($name:ident, $input:literal) => {
        #[test]
        fn $name() {
            $crate::common::util::assert_snapshot!($crate::common::util::mock_interpret($input));
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! parse {
    ($name:ident, $input:literal) => {
        paste::paste!(
            #[test]
            fn [<_parse_test_ $name>]() {
                $crate::common::util::assert_snapshot!($crate::common::util::mock_parse($input));
            }
        );
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! snap_success {
    ($name:ident, $input: literal) => {
        $crate::snap!($name, $input);
        $crate::parse!($name, $input);
    }
}
