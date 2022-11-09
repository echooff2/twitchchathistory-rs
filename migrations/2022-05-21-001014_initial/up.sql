CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE channels (
    id SERIAL PRIMARY KEY NOT NULL,
    uuid UUID UNIQUE NOT NULL DEFAULT uuid_generate_v4(),
    twitch_channel_id VARCHAR NOT NULL,
    channel_name VARCHAR NOT NULL
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY NOT NULL,
    uuid UUID UNIQUE NOT NULL DEFAULT uuid_generate_v4(),
    username VARCHAR NOT NULL,
    twitch_user_id VARCHAR UNIQUE NOT NULL
);

CREATE TABLE resubs (
    id SERIAL PRIMARY KEY NOT NULL,
    uuid UUID UNIQUE NOT NULL DEFAULT uuid_generate_v4(),
    cumulative_month SMALLINT NOT NULL,
    tier SMALLINT NOT NULL -- 0 == prime
);

CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    uuid UUID UNIQUE NOT NULL DEFAULT uuid_generate_v4(),
    msg VARCHAR NOT NULL,
    msg_type VARCHAR NOT NULL, -- message/bits/sub/action
    user_id INTEGER NOT NULL,
    channel_id INTEGER NOT NULL,
    resub_id INTEGER UNIQUE, -- null if not resub
    send_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bits BIGINT,

    CONSTRAINT FK_messages_users FOREIGN KEY(user_id)
        REFERENCES users(id),

    CONSTRAINT FK_messages_channels FOREIGN KEY(channel_id)
        REFERENCES channels(id),

    CONSTRAINT FK_messages_resubs FOREIGN KEY(resub_id)
        REFERENCES resubs(id)
);

CREATE INDEX messages_user_id_idx ON messages ( user_id );
CREATE INDEX messages_channel_id_idx ON messages ( channel_id );
CREATE INDEX messages_resub_id_idx ON messages ( resub_id );

CREATE TABLE users_old_names (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    username VARCHAR NOT NULL,
    first_time_with_new_name TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT FK_users_old_names_users FOREIGN KEY(user_id)
        REFERENCES users(id)
);
