#[derive(PartialEq)]
#[non_exhaustive]
pub enum Mode {
    Draw,
    Erase,
    AddPinkConstraint,
    RemovePinkConstraint,
    StartSelection,
    EndSelection,
}