CREATE TABLE shorts (
    id SERIAL PRIMARY KEY,
    short VARCHAR(16) UNIQUE,
    long_url text NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE shorts_goto_stats (
    id SERIAL PRIMARY KEY,
    short_id INTEGER REFERENCES shorts(id),
    goto_at TIMESTAMPTZ DEFAULT NOW()
);