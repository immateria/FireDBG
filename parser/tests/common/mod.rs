#![allow(
    unused_imports,
    unused_variables,
    unused_assignments,
    dead_code,
    clippy::never_loop,
    clippy::approx_constant,
    // These files double as parser fixtures (span-sensitive) and as test modules.
    // Prefer lint suppression here over editing the fixture sources and drifting spans.
    clippy::needless_lifetimes,
    clippy::unused_unit,
    clippy::needless_late_init,
    clippy::assign_op_pattern,
    clippy::no_effect
)]

pub mod free_fn;
pub mod impl_fn;
pub mod impl_trait;
pub mod nested_fn;
pub mod result_fn;
pub mod stmt;
