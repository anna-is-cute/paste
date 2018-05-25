alter table email_verifications add column expiry timestamp not null default now() + '1 hour';

create function email_verifications_delete_old_rows() returns trigger
  language plpgsql
  as $$
begin
  delete from email_verifications where expiry < now();
  return new;
end;
$$;

create trigger email_verifications_delete_old_rows_trigger
  after insert on email_verifications
  execute procedure email_verifications_delete_old_rows();
