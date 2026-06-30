-- This file should undo anything in `up.sql`
ALTER TABLE orders
    ALTER COLUMN status
    TYPE text
    USING status::text;

DROP TYPE dborderstatus;

CREATE TYPE dborderstatus AS ENUM (
    'New', 'InProgress', 'ReadyToShip', 'Shipping', 'Done'
);

ALTER TABLE orders
    ALTER COLUMN status
    TYPE dborderstatus
    USING status::dborderstatus;
