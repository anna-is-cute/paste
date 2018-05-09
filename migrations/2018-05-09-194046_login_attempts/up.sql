create table login_attempts (
  addr cidr primary key,
  timestamp timestamp not null default now(),
  attempts integer not null
);

create function login_attempts_delete_old_rows() returns trigger
  language plpgsql
  as $$
begin
  delete from login_attempts where timestamp < now() - interval '30 minutes';
  return new;
end;
$$;

create trigger login_attempts_delete_old_rows_trigger
  after insert on login_attempts
  execute procedure login_attempts_delete_old_rows();
