-- Baseline migration for existing tables.
-- DDL reconstructed from the running database to ensure new environments
-- match the current schema exactly.

CREATE TABLE IF NOT EXISTS rt_app (
    id BIGSERIAL PRIMARY KEY,
    app_id VARCHAR(100) NOT NULL,
    app_key VARCHAR(200) NOT NULL,
    expire_time TIMESTAMPTZ,
    created_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_time TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS rt_client_connection (
    id BIGSERIAL PRIMARY KEY,
    app_id BIGINT NOT NULL DEFAULT 0,
    rt_app VARCHAR(100) NOT NULL DEFAULT '',
    client_id VARCHAR(100) NOT NULL DEFAULT '',
    connect_token VARCHAR(100) NOT NULL DEFAULT '',
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expire_time TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_app_id ON rt_client_connection(app_id);
CREATE INDEX IF NOT EXISTS idx_created_time ON rt_client_connection(created_time);
CREATE INDEX IF NOT EXISTS idx_expire_time ON rt_client_connection(expire_time);
