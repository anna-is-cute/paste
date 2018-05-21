create table users (
  id uuid primary key not null,
  username text not null,
  password text not null
);

create table api_keys (
  key uuid primary key not null,
  user_id uuid not null,

  foreign key (user_id) references users(id) on delete cascade
);

create table pastes (
  id uuid primary key not null,
  name text,
  visibility smallint not null,
  author_id uuid,

  foreign key (author_id) references users(id) on delete cascade
);

create table deletion_keys (
  key uuid primary key not null,
  paste_id uuid not null,

  foreign key (paste_id) references pastes(id) on delete cascade
);

create table files (
  id uuid primary key not null,
  paste_id uuid not null,
  name text,
  is_binary boolean,

  foreign key (paste_id) references pastes(id) on delete cascade
);
