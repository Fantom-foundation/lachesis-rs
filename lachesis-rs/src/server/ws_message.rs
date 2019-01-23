use actix::*;

use bytes::Bytes;

use actix_web::Binary;
use bincode::serialize;

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum InternodeMessage {
    SyncRequest,
    SyncResponse,
}

impl Into<Binary> for InternodeMessage {
    fn into(self) -> Binary {
        match serialize(&self) {
            Ok(encoded) => Binary::Bytes(Bytes::from(encoded)),
            Err(e) => {
                error!("{}", e);
                Binary::Bytes(Bytes::from(vec![]))
            }
        }
    }
}
