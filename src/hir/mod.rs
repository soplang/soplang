//! High-level IR (HIR): backend-agnostic representation.

mod lower;

pub use lower::{
    BinOpKind, HirConst, HirFunction, HirInstr, HirLowering, HirModule, LabelId, Slot, SlotNum,
    UnOpKind,
};
