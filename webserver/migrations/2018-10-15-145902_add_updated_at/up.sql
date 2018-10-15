create function update_updated_at_pastes() returns trigger as $$
begin
  NEW.updated_at = now();
  return NEW;
end; $$
language plpgsql;

alter table pastes
  add column updated_at timestamp;

create trigger update_updated_at_pastes_trigger
  before update on pastes
  for each row
  execute procedure update_updated_at_pastes();
