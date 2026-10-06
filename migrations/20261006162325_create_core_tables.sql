CREATE TYPE user_role AS ENUM
    ('owner', 'admin', 'hr', 'finance', 'supervisor', 'driver');

CREATE TABLE districts (
    id             SERIAL PRIMARY KEY,
    name           TEXT NOT NULL,
    state          TEXT NOT NULL DEFAULT 'Lagos',
    collection_day TEXT NOT NULL,
    UNIQUE (name, state)
);

CREATE TABLE users (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name  TEXT NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    phone      TEXT,
    role       user_role NOT NULL,
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE trucks (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fleet_code    TEXT NOT NULL UNIQUE,
    driver_id     UUID REFERENCES users(id),
    supervisor_id UUID REFERENCES users(id),
    shift_start   SMALLINT NOT NULL CHECK (shift_start BETWEEN 0 AND 23),
    shift_end     SMALLINT NOT NULL CHECK (shift_end BETWEEN 0 AND 23),
    CHECK (shift_start <> shift_end)
);

CREATE TABLE routes (
    id          SERIAL PRIMARY KEY,
    truck_id    UUID NOT NULL REFERENCES trucks(id) ON DELETE CASCADE,
    district_id INT NOT NULL REFERENCES districts(id),
    street      TEXT NOT NULL,
    lane        SMALLINT NOT NULL CHECK (lane > 0),
    first_house INT NOT NULL,
    last_house  INT NOT NULL,
    CHECK (first_house <= last_house)
);
