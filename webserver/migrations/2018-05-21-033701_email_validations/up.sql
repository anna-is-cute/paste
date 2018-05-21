create table email_verifications (
  id uuid primary key,
  email text not null,
  user_id uuid not null,
  key uuid not null,
  last_sent timestamp,

  foreign key (user_id) references users(id)
);

alter table users add column email_verified boolean not null default false;
