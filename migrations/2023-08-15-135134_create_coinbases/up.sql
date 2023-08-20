-- Your SQL goes here
create table
  coinbases (
    id character varying(255) not null,
    height BIGINT null,
    da_height BIGINT null,
    block_hash character varying(255) not null,
    amount BIGINT null,
    coinbase character varying(255) null,
    timestamp BIGINT,
    constraint coinbases_pkey primary key (id)
  ) tablespace pg_default;