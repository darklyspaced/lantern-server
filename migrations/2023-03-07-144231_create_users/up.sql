CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  email VARCHAR UNIQUE NOT NULL,
  firefly_secret VARCHAR NOT NULL,
device_id VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
  id SERIAL PRIMARY KEY,
	user_email VARCHAR UNIQUE NOT NULL,
  	CONSTRAINT fk_email
    	FOREIGN KEY (user_email) REFERENCES users(email),
    local_tasks JSON NOT NULL,
    firefly_tasks JSON NOT NULL
);
