use crate::field::cell::CellType;

#[derive(PartialEq)]
#[non_exhaustive]
pub enum Mode {
    Draw(CellType),
    Erase,
    StartSelection,
    EndSelection,
}
