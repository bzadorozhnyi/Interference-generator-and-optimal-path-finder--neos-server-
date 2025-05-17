pub enum NeosResponse {
    Message(String),
    Error(String),
    JobCredentials(i32, String),
    JobOuput(String)
}