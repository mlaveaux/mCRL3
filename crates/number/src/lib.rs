#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod bits_for_value;
mod power_of_two;
mod u64_variablelength;

pub use bits_for_value::bits_for_value;
pub use power_of_two::is_power_of_two;
pub use power_of_two::round_up_to_power_of_two;
pub use u64_variablelength::read_u64_variablelength;
pub use u64_variablelength::write_u64_variablelength;
