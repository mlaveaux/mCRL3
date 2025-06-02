/// A macro to return the pat type of an enum class target, and panics otherwise.
///
/// Usage cast!(instance, type)
#[macro_export]
macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat));
        }
    }};
}
