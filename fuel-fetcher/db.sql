create table price (
    state int not null,
    station int not null,
    fuel int not null,
    updated_at int not null,
    price numeric,
    primary key (state, station, fuel)
);

create table price_history (
    state int not null,
    station int not null,
    fuel int not null,
    changed_at int not null,
    price numeric
);

create index price_history_index on price_history (state, station, fuel);
