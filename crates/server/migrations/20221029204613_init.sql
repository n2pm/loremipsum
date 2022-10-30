create extension if not exists timescaledb;

create table seen_servers (id varchar(48) primary key not null);

create table seen_players (
  id varchar(48) primary key not null,
  valid boolean default null
);

create table seen_servers_data (
  time timestamp with time zone not null,
  id varchar(48) references seen_servers not null,
  motd varchar(4096),
  favicon bytea,
  current_player_count integer not null,
  max_player_count integer not null,
  server_version varchar(256),
  protocol_version integer not null
);

select
  create_hypertable('seen_servers_data', 'time');

create table seen_players_data (
  time timestamp with time zone not null,
  id varchar(36) references seen_players not null,
  name varchar(32),
  connected_to varchar(32) references seen_servers
);

select
  create_hypertable('seen_players_data', 'time');

create table api_keys (
  key varchar(64) primary key not null,
  admin boolean not null default false
);

create type job_type as enum ('ping_server', 'get_player', 'test_whitelist');

create table pending_jobs (
  id serial,
  time timestamp with time zone not null,
  type job_type not null,
  data varchar(64) not null
);