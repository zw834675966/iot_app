DELETE FROM routes
WHERE path = '/permission/button'
   OR path LIKE '/permission/button/%';
