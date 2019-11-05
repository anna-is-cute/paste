alter table deletion_keys alter key type uuid using key::uuid;
alter table deletion_keys drop constraint deletion_keys_pkey;
alter table deletion_keys add primary key (key);
