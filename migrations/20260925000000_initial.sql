-- Keep existing serial IDs and bcrypt hashes when migrating an Express database.
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS summaries (
    id SERIAL PRIMARY KEY,
    original TEXT NOT NULL,
    summary TEXT NOT NULL,
    user_id INTEGER REFERENCES users(id),
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC')
);

-- Old rows without an owner stay unassigned.
ALTER TABLE summaries ADD COLUMN IF NOT EXISTS user_id INTEGER;
ALTER TABLE summaries ADD COLUMN IF NOT EXISTS created_at TIMESTAMP;
UPDATE summaries SET created_at = now() AT TIME ZONE 'UTC' WHERE created_at IS NULL;
ALTER TABLE summaries ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE summaries ALTER COLUMN created_at SET DEFAULT (now() AT TIME ZONE 'UTC');

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'summaries'::regclass
          AND conname = 'summaries_user_id_fkey'
    ) THEN
        ALTER TABLE summaries
            ADD CONSTRAINT summaries_user_id_fkey
            FOREIGN KEY (user_id) REFERENCES users(id);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS summaries_user_history_idx
    ON summaries (user_id, created_at DESC, id DESC);
