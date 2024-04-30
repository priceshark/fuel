create table price (
    state text not null,
    station int not null,
    fuel text not null,
    price float,
    changed_at int not null,
    checked_at int not null,
    primary key (state, station, fuel)
);
