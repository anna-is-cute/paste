create table password_resets (
  id uuid primary key,
  secret text not null,
  expiry timestamp not null,
  user_id uuid not null,

  foreign key (user_id) references users(id)
)
