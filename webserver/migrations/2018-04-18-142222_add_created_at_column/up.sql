alter table pastes add column created_at timestamp not null default now();
alter table files add column created_at timestamp not null default now();
