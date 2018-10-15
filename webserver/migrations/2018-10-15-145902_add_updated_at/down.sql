alter table pastes drop column updated_at;

drop trigger update_updated_at_pastes_trigger on pastes;

drop function update_updated_at_pastes;
