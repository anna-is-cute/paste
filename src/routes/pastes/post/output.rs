use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Success {
  id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  deletion_key: Option<String>,
}

impl Success {
  pub fn new(id: Uuid, deletion_key: Option<Uuid>) -> Self {
    let id = id.simple().to_string();
    let deletion_key = deletion_key.map(|x| x.simple().to_string());
    Success { id, deletion_key }
  }
}
