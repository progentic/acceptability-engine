ALTER TABLE contracts ADD COLUMN candidate_sha TEXT;

UPDATE contracts
SET candidate_sha = base_sha
WHERE candidate_sha IS NULL OR candidate_sha = '';

ALTER TABLE contracts ADD COLUMN candidate_ref TEXT;

ALTER TABLE contracts ADD COLUMN scopes_json TEXT NOT NULL DEFAULT '[]';
