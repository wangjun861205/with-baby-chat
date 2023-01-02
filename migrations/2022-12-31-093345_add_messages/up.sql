-- Your SQL goes here
CREATE TABLE messages (
    id SERIAL NOT NULL PRIMARY KEY,
    "from" INT NOT NULL REFERENCES users(id),
    "to" INT NOT NULL REFERENCES users(id),
    content VARCHAR NOT NULL
);