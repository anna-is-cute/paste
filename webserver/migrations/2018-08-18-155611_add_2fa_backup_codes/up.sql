create table backup_codes (
  user_id uuid not null,
  code varchar(12) not null,

  primary key (user_id, code),
  foreign key (user_id) references users(id)
)
