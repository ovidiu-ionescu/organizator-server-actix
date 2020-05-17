UPDATE users
SET salt=$1, pbkdf2 = $2
WHERE username = $3;