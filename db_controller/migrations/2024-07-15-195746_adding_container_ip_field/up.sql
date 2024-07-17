-- Your SQL goes here
DROP TABLE active_sessions;

CREATE TABLE active_sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  user VARCHAR(255) NOT NULL,
  service_id INT NOT NULL,
  docker_id VARCHAR(255),
  container_ip VARCHAR(255),
  port INT,
  unix_created_at BIGINT NOT NULL,
  FOREIGN KEY ("user") REFERENCES "users" ("username") ON DELETE CASCADE,
  FOREIGN KEY ("service_id") REFERENCES "services"("id") ON DELETE CASCADE
);