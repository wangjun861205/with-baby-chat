DROP TABLE IF EXISTS join_applications CASCADE;

CREATE TABLE join_applications (
	id SERIAL NOT NULL PRIMARY KEY,
	"from" INT NOT NULL REFERENCES users(id),
	"to" INT NOT NULL REFERENCES channels(id),
	UNIQUE("from", "to")
);