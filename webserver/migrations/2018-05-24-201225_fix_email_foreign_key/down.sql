alter table email_verifications drop constraint email_verifications_user_id_fkey;

alter table email_verifications add constraint email_verifications_user_id_fkey
  foreign key (user_id) references users(id);

alter table password_resets drop constraint password_resets_user_id_fkey;

alter table password_resets add constraint password_resets_user_id_fkey
  foreign key (user_id) references users(id);
