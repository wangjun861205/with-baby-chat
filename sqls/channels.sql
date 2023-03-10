DROP TABLE IF EXISTS channels CASCADE;

CREATE TABLE channels (
	id SERIAL NOT NULL PRIMARY KEY,
	name VARCHAR NOT NULL,
	description VARCHAR NOT NULL,
	administrator INT NOT NULL REFERENCES users(id),
	UNIQUE(name)
);