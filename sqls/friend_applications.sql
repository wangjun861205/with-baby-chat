DROP TABLE IF EXISTS friend_applications CASCADE;

CREATE TABLE friend_applications (
	id SERIAL NOT NULL PRIMARY KEY,
	"from" INT NOT NULL REFERENCES users(id),
	"to" INT NOT NULL REFERENCES users(id),
	UNIQUE("from", "to")
)