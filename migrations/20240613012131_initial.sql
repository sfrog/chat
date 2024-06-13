-- Add migration script here

-- create users table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for users.email
CREATE UNIQUE INDEX IF NOT EXISTS users_email_idx ON users(email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create chats table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    type chat_type NOT NULL,
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create messages table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    images TEXT[],
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages.chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS messages_chat_id_created_at_idx ON messages(chat_id, created_at DESC);

-- create index for messages.sender_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS messages_sender_id_idx ON messages(sender_id, created_at DESC);
