mod common;

pub use firedbg_rust_parser::*;
pub use pretty_assertions::assert_eq;

fn normalize_end_columns(bps: &mut [FunctionDef]) {
    for bp in bps.iter_mut() {
        bp.end.column = None;
    }
}

#[test]
fn parse_nested_fn() -> anyhow::Result<()> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/nested_fn.rs");
    let mut breakpoints = parse_file(path)?;
    breakpoints.pop(); // Pop the last `fn get_breakpoints` breakpoint

    let mut expected = common::nested_fn::get_breakpoints();
    normalize_end_columns(&mut breakpoints);
    normalize_end_columns(&mut expected);

    assert_eq!(breakpoints, expected);
    Ok(())
}
