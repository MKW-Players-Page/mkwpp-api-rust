CREATE TYPE region_type AS ENUM ('world', 'country_group', 'country', 'subnational_group', 'subnational');
CREATE TABLE regions (
    id SERIAL PRIMARY KEY,
    code VARCHAR(32) NOT NULL,
    type region_type,
    parent_id INTEGER REFERENCES regions(id),
    is_ranked BOOL DEFAULT FALSE
);

CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    alias VARCHAR(64),
    bio VARCHAR(1024),
    region_id INTEGER REFERENCES regions(id),
    joined_date DATE NOT NULL DEFAULT NOW(),
    last_activity DATE NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(150) NOT NULL,
    password VARCHAR(128) NOT NULL,
    email VARCHAR(254) NOT NULL,
    last_login TIMESTAMP WITH TIME ZONE,
    is_superuser BOOL DEFAULT FALSE NOT NULL,
    is_staff BOOL DEFAULT FALSE NOT NULL,
    is_active BOOL DEFAULT FALSE NOT NULL,
    is_verified BOOL DEFAULT FALSE NOT NULL,
    date_joined TIMESTAMP WITH TIME ZONE NOT NULL,
    player_id INTEGER REFERENCES players(id)
);

-- nonsc = 0
-- sc = 1
-- unres = 2

CREATE TYPE category AS ENUM ('nonsc', 'sc', 'unres');

CREATE TABLE tracks (
    id SERIAL PRIMARY KEY,
    abbr VARCHAR(16) NOT NULL,
    cup_id INTEGER NOT NULL,
    categories category[]
);

CREATE TABLE scores (
    id SERIAL PRIMARY KEY,
    value INTEGER NOT NULL,
    category category NOT NULL,
    is_lap BOOL NOT NULL,
    player_id INTEGER NOT NULL REFERENCES players(id),
    track_id INTEGER NOT NULL REFERENCES tracks(id),
    date DATE,
    video_link VARCHAR(255),
    ghost_link VARCHAR(255),
    comment VARCHAR(128),
    admin_note VARCHAR(255),
    initial_rank INTEGER
);

-- pending = 0
-- accepted = 1
-- rejected = 2
-- on_hold = 3

CREATE TYPE submission_status AS ENUM ('pending', 'accepted', 'rejected', 'on_hold');

CREATE TABLE submissions (
    id SERIAL PRIMARY KEY,
    value INTEGER NOT NULL,
    category category NOT NULL,
    is_lap BOOL NOT NULL,
    player_id INTEGER NOT NULL REFERENCES players(id),
    track_id INTEGER NOT NULL REFERENCES tracks(id),
    date DATE,
    video_link VARCHAR(255),
    ghost_link VARCHAR(255),
    comment VARCHAR(128),
    admin_note VARCHAR(255),
    status submission_status,
    submitter_id INTEGER NOT NULL REFERENCES users(id),
    submitter_note VARCHAR(255),
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reviewer_id INTEGER NOT NULL REFERENCES users(id),
    reviewer_note VARCHAR(255),
    reviewed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    score_id INTEGER
);

CREATE TABLE editsubmission (
    id SERIAL PRIMARY KEY,
    date DATE,
    video_link VARCHAR(255),
    video_link_edited BOOL DEFAULT FALSE NOT NULL,
    ghost_link VARCHAR(255),
    ghost_link_edited BOOL DEFAULT FALSE NOT NULL,
    comment VARCHAR(128),
    comment_edited BOOL DEFAULT FALSE NOT NULL,
    admin_note VARCHAR(255),
    status submission_status,
    submitter_id INTEGER NOT NULL REFERENCES users(id),
    submitter_note VARCHAR(255),
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reviewer_id INTEGER NOT NULL REFERENCES users(id),
    reviewer_note VARCHAR(255),
    reviewed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    score_id INTEGER NOT NULL REFERENCES scores(id)
);

CREATE TABLE standardlevels (
    id SERIAL PRIMARY KEY,
    code VARCHAR(32) NOT NULL,
    value INTEGER NOT NULL,
    is_legacy BOOL NOT NULL
);

CREATE TABLE standards (
    id SERIAL PRIMARY KEY,
    value INTEGER NOT NULL,
    standardlevel_id INTEGER NOT NULL REFERENCES standardlevels(id),
    track_id INTEGER NOT NULL REFERENCES tracks(id),
    category category NOT NULL,
    is_lap BOOL NOT NULL
);

CREATE TABLE sitechamps (
    id SERIAL PRIMARY KEY,
    player_id INTEGER NOT NULL REFERENCES players(id),
    category category NOT NULL,
    date_instated DATE NOT NULL
);

CREATE TYPE playerawardtype AS ENUM ('weekly', 'monthly', 'quarterly', 'yearly');

CREATE TABLE playerawards (
    id SERIAL PRIMARY KEY,
    player_id INTEGER NOT NULL REFERENCES players(id),
    type playerawardtype NOT NULL,
    date DATE NOT NULL,
    description VARCHAR(1024)
);
