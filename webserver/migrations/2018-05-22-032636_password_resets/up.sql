create table password_resets (
  id uuid primary key,
  secret text not null,
  expiry timestamp not null,
  user_id uuid not null,

  foreign key (user_id) references users(id)
);

create function password_resets_delete_old_rows() returns trigger
  language plpgsql
  as $$
begin
  delete from password_resets where expiry < now();
  return new;
end;
$$;

create trigger password_resets_delete_old_rows_trigger
  after insert on password_resets
  execute procedure password_resets_delete_old_rows();
