use super::super::schema::backup_codes;
use super::users::User;
use crate::models::id::UserId;

#[derive(Debug, Serialize, Identifiable, Queryable, Associations)]
#[primary_key(user_id, code)]
#[belongs_to(User)]
pub struct BackupCode {
  user_id: UserId,
  code: String,
}

#[derive(Insertable)]
#[table_name = "backup_codes"]
pub struct NewBackupCode {
  user_id: UserId,
  code: String,
}

impl NewBackupCode {
  pub fn new(user_id: UserId, code: String) -> Self {
    NewBackupCode { user_id, code }
  }
}
