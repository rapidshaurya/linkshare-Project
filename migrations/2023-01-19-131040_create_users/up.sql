-- Your SQL goes here
-- Your SQL goes here
-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

create table persons(
id uuid default uuid_generate_v4(),
username varchar not null unique,
password varchar not null,
primary key (id)
)