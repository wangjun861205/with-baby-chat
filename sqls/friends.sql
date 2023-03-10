DROP TABLE IF EXISTS friends CASCADE;

CREATE TABLE friends (
	id SERIAL NOT NULL PRIMARY KEY,
	user_a INT NOT NULL REFERENCES users(id),
	user_b INT NOT NULL REFERENCES users(id),
	UNIQUE(user_a, user_b)
);