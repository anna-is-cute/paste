alter table email_verifications drop column expiry;

drop trigger email_verifications_delete_old_rows_trigger on email_verifications;

drop function email_verifications_delete_old_rows;
