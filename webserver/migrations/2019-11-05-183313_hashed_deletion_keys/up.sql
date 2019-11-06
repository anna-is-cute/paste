alter table deletion_keys alter key type text using replace(key::text, '-', '');
alter table deletion_keys drop constraint deletion_keys_pkey;
alter table deletion_keys add primary key (paste_id);
