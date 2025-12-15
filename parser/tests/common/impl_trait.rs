use async_trait::async_trait;

#[async_trait]
pub trait TraitA {
    async fn func_default<T: Into<i32> + Send>(i: T) -> &'static str {
        "TraitA::func_default()"
    }

    fn func_required() -> &'static str;
}

pub trait TraitB {
    // fn func_self_required(&self) -> impl std::future::Future<Output = &'static str>; // FIXME: Should this be parsed as async function?
    fn func_self_required(&self) -> &'static str;
}

pub struct StructA;

#[async_trait]
impl TraitA for StructA {
    fn func_required() -> &'static str {
        "<StructA as TraitA>::func_required()"
    }
}

impl TraitB for StructA {
    fn func_self_required(&self) -> &'static str {
        "<StructA as TraitB>::func_self_required()"
    }
}

pub struct StructB {
    field_1: i32,
    pub field_2: usize,
    pub(crate) field_3: u64,
}

#[async_trait]
impl TraitA for StructB {
    async fn func_default<T>(i: T) -> &'static str
    where
        T: Into<i32> + Send,
    {
        "<StructB as TraitA>::func_default()"
    }

    fn func_required() -> &'static str {
        "<StructB as TraitA>::func_required()"
    }
}

impl TraitB for StructB {
    fn func_self_required(&self) -> &'static str {
        "<StructB as TraitB>::func_self_required()"
    }
}

pub struct StructC(i32, pub usize, pub(crate) u64);

#[async_trait]
impl TraitA for StructC {
    fn func_required() -> &'static str {
        "<StructC as TraitA>::func_required()"
    }
}

impl TraitB for StructC {
    fn func_self_required(&self) -> &'static str {
        "<StructC as TraitB>::func_self_required()"
    }
}

mod module_a {
    use super::*;

    pub trait TraitC {
        fn func_default<T: Into<i32> + Send>(i: T) -> &'static str {
            "TraitC::func_default()"
        }

        fn func_required() -> &'static str;
    }

    pub trait TraitD {
        fn func_self_required(&self) -> &'static str;
    }

    pub struct StructD;

    #[async_trait]
    impl TraitA for StructD {
        fn func_required() -> &'static str {
            "<StructD as TraitA>::func_required()"
        }
    }

    impl TraitB for StructD {
        fn func_self_required(&self) -> &'static str {
            "<StructD as TraitB>::func_self_required()"
        }
    }

    impl TraitD for super::StructA {
        fn func_self_required(&self) -> &'static str {
            "<super::StructA as TraitD>::func_self_required()"
        }
    }
}

use firedbg_protocol::source::*;
use firedbg_rust_parser::*;

pub fn get_breakpoints() -> Vec<FunctionDef> {
    vec![
        FunctionDef {
            ty: FunctionType::TraitDefaultFn {
                trait_name: "TraitA".into(),
                fn_name: "func_default".into(),
                is_async: true,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 5,
                    column: Some(71),
                },
                end: LineColumn {
                    line: 6,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 7,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitA".into(),
                self_type: "StructA".into(),
                fn_name: "func_required".into(),
                is_async: false,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 21,
                    column: Some(41),
                },
                end: LineColumn {
                    line: 22,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 23,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitB".into(),
                self_type: "StructA".into(),
                fn_name: "func_self_required".into(),
                is_async: false,
                is_static: false,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 27,
                    column: Some(51),
                },
                end: LineColumn {
                    line: 28,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 29,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitA".into(),
                self_type: "StructB".into(),
                fn_name: "func_default".into(),
                is_async: true,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 43,
                    column: Some(6),
                },
                end: LineColumn {
                    line: 44,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 45,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitA".into(),
                self_type: "StructB".into(),
                fn_name: "func_required".into(),
                is_async: false,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 47,
                    column: Some(41),
                },
                end: LineColumn {
                    line: 48,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 49,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitB".into(),
                self_type: "StructB".into(),
                fn_name: "func_self_required".into(),
                is_async: false,
                is_static: false,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 53,
                    column: Some(51),
                },
                end: LineColumn {
                    line: 54,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 55,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitA".into(),
                self_type: "StructC".into(),
                fn_name: "func_required".into(),
                is_async: false,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 62,
                    column: Some(41),
                },
                end: LineColumn {
                    line: 63,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 64,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitB".into(),
                self_type: "StructC".into(),
                fn_name: "func_self_required".into(),
                is_async: false,
                is_static: false,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 68,
                    column: Some(51),
                },
                end: LineColumn {
                    line: 69,
                    column: Some(9),
                },
            },
            end: LineColumn {
                line: 70,
                column: Some(4),
            },
        },
        FunctionDef {
            ty: FunctionType::TraitDefaultFn {
                trait_name: "TraitC".into(),
                fn_name: "func_default".into(),
                is_async: false,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 77,
                    column: Some(69),
                },
                end: LineColumn {
                    line: 78,
                    column: Some(13),
                },
            },
            end: LineColumn {
                line: 79,
                column: Some(8),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitA".into(),
                self_type: "StructD".into(),
                fn_name: "func_required".into(),
                is_async: false,
                is_static: true,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 92,
                    column: Some(45),
                },
                end: LineColumn {
                    line: 93,
                    column: Some(13),
                },
            },
            end: LineColumn {
                line: 94,
                column: Some(8),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitB".into(),
                self_type: "StructD".into(),
                fn_name: "func_self_required".into(),
                is_async: false,
                is_static: false,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 98,
                    column: Some(55),
                },
                end: LineColumn {
                    line: 99,
                    column: Some(13),
                },
            },
            end: LineColumn {
                line: 100,
                column: Some(8),
            },
        },
        FunctionDef {
            ty: FunctionType::ImplTraitFn {
                trait_name: "TraitD".into(),
                self_type: "super :: StructA".into(),
                fn_name: "func_self_required".into(),
                is_async: false,
                is_static: false,
                return_type: true,
            },
            loc: BreakableSpan {
                start: LineColumn {
                    line: 104,
                    column: Some(55),
                },
                end: LineColumn {
                    line: 105,
                    column: Some(13),
                },
            },
            end: LineColumn {
                line: 106,
                column: Some(8),
            },
        },
    ]
}
