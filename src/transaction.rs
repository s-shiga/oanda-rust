use serde::{Deserialize, Serialize};

pub type TransactionId = String;

pub type ClientID = String;

pub type ClientTag = String;

pub type ClientComment = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientExtensions {
    pub id: ClientID,
    pub tag: ClientTag,
    pub comment: ClientComment,
}
