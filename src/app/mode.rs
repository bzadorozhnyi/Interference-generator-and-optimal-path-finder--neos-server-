#[derive(PartialEq)]
#[non_exhaustive]
pub enum Mode {
    Draw,
    Erase,
    StartSelection,
    EndSelection,
}