pub enum NeosResponse {
    Message(String),
    JobCredentials(i32, String),
    JobOuput(String)
}