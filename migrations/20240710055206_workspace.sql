-- Add migration script here

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL,
    owner_id BIGINT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- alter users table to add workspace_id
ALTER TABLE users
  ADD COLUMN ws_id BIGINT REFERENCES workspaces(id);

-- add super user 0 and workspace 0
BEGIN;
INSERT INTO users (id, fullname, email, password_hash)
  VALUES (0, 'superuser', 'superuser@none.org', '');
INSERT INTO workspaces (id, name, owner_id)
  VALUES (0, 'default', 0);
UPDATE users SET ws_id = 0 WHERE id = 0;
COMMIT;

-- alter user table to make ws_id not null
ALTER TABLE users
  ALTER COLUMN ws_id SET NOT NULL;
