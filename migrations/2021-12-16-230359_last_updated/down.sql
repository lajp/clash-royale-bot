-- This file should undo anything in `up.sql`
ALTER TABLE Players
DROP COLUMN last_updated;
