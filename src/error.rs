#[derive(Debug)]
pub enum AppError {
    InvalidPath,
    ParseStringError(String),
    StartNotSet,
    EndNotSet,
    FailedRenderFile,
    InvalidAuthCredentials,
    FailedUpdateConfig,
    FailedTakeScreenshot,
}
