-- Add up migration script here
CREATE TABLE IF NOT EXISTS hosts (
  id BIGSERIAL PRIMARY KEY,
  service_id BIGSERIAL NOT NULL,
  ip TEXT NOT NULL,
  port SMALLINT NOT NULL,
  FOREIGN KEY (service_id) REFERENCES services(id)
)
  
