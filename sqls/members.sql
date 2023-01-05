DROP TABLE IF EXISTS members CASCADE;

CREATE TABLE members (
	id SERIAL NOT NULL PRIMARY KEY,
	"user" INT NOT NULL REFERENCES users(id),
	channel INT NOT NULL REFERENCES channels(id),
	UNIQUE ("user", channel)
);