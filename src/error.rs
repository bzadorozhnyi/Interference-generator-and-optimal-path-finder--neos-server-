pub enum FieldError {
    InvalidPath,
    ParseStringError(String),
    StartNotSet,
    EndNotSet
}