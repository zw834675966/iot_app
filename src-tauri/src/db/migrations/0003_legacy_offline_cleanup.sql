UPDATE users
SET avatar = ''
WHERE TRIM(avatar) != ''
  AND (
    LOWER(TRIM(avatar)) LIKE 'http://%'
    OR LOWER(TRIM(avatar)) LIKE 'https://%'
    OR LOWER(TRIM(avatar)) LIKE '//%'
    OR POSITION('://' IN LOWER(TRIM(avatar))) > 0
  );

UPDATE routes
SET meta_icon = NULL
WHERE meta_icon IS NOT NULL
  AND TRIM(meta_icon) != ''
  AND (
    POSITION(':' IN meta_icon) > 0
    OR LOWER(TRIM(meta_icon)) LIKE 'http://%'
    OR LOWER(TRIM(meta_icon)) LIKE 'https://%'
    OR LOWER(TRIM(meta_icon)) LIKE '//%'
    OR POSITION('://' IN LOWER(TRIM(meta_icon))) > 0
  );
