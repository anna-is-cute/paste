alter table users
  add column shared_secret bytea,
  add column tfa_enabled boolean not null default false

