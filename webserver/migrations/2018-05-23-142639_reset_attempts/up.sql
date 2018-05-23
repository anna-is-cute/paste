create table password_reset_attempts (
  addr cidr primary key,
  timestamp timestamp not null default now(),
  attempts integer not null
);

create function password_reset_attempts_delete_old_rows() returns trigger
  language plpgsql
  as $$
begin
  delete from password_reset_attempts where timestamp < now() - interval '1 hour';
  return new;
end;
$$;

create trigger password_reset_attempts_delete_old_rows_trigger
  after insert on password_reset_attempts
  execute procedure password_reset_attempts_delete_old_rows();
