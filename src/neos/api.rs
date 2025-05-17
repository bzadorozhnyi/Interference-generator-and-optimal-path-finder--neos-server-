use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use dxr_client::{Client, ClientBuilder, Url};

use crate::consts::NEOS_API_URL;

use super::response::NeosResponse;

pub struct NeosAPI {
    client: Arc<Client>,
    tx: Sender<NeosResponse>,
    pub rx: Receiver<NeosResponse>,
    pub response: String,
}

impl NeosAPI {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let url = Url::parse(NEOS_API_URL).unwrap();

        Self {
            client: Arc::new(ClientBuilder::new(url).build()),
            tx,
            rx,
            response: String::new(),
        }
    }

    fn clone_client_tx(&self) -> (Arc<Client>, Sender<NeosResponse>) {
        (Arc::clone(&self.client), self.tx.clone())
    }

    pub fn ping(&self) {
        let (client, tx) = self.clone_client_tx();

        tokio::spawn(async move {
            let response: Result<String, dxr_client::ClientError> = client.call("ping", ()).await;
            if let Ok(body) = response {
                let _ = tx.send(NeosResponse::Message(body));
            }
        });
    }

    pub fn submit_job(&self, input: String) {
        let (client, tx) = self.clone_client_tx();

        tokio::spawn(async move {
            let response: Result<(i32, String), dxr_client::ClientError> =
                client.call("submitJob", input).await;

            if let Ok(body) = response {
                match body {
                    (0, error_msg) => {
                        let _ = tx.send(NeosResponse::Message(error_msg));
                    }
                    (job_number, job_password) => {
                        let _ = tx.send(NeosResponse::JobCredentials(job_number, job_password));
                    }
                }
            }
        });
    }

    pub fn get_final_results(&self, job_number: i32, job_password: String) {
        let (client, tx) = self.clone_client_tx();

        tokio::spawn(async move {
            let response: Result<Vec<u8>, dxr_client::ClientError> = client
                .call("getFinalResults", (job_number, job_password))
                .await;

            if let Ok(body) = response {
                let output = String::from_utf8(body).unwrap();
                let _ = tx.send(NeosResponse::JobOuput(output));
            }
        });
    }
}

impl Default for NeosAPI {
    fn default() -> Self {
        Self::new()
    }
}
