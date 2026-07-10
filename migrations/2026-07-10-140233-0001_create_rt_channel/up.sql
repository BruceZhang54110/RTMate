CREATE TABLE rt_channel (
    id BIGSERIAL PRIMARY KEY,
    app_id BIGINT NOT NULL,
    app_id_str VARCHAR(100) NOT NULL,
    name VARCHAR(64) NOT NULL,
    description VARCHAR(256),
    created_by VARCHAR(100),
    created_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_time TIMESTAMPTZ,

    CONSTRAINT uq_rt_channel_app_id_name
        UNIQUE (app_id, name)
);

CREATE INDEX idx_rt_channel_app_id_str ON rt_channel(app_id_str);
