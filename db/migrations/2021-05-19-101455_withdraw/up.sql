CREATE SEQUENCE withdraw_seq;

CREATE TABLE IF NOT EXISTS withdraw (
    idx INT NOT NULL DEFAULT nextval('withdraw_seq'),
    state VARCHAR NOT NULL,
    sig_count INT NOT NULL,
    height INT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (idx)
);
