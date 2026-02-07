CREATE TYPE orderstatus AS ENUM (
    'New', 'InProgress', 'ReadyToShip', 'Shipping', 'Done'
);

ALTER TABLE orders
    ALTER COLUMN status
    TYPE orderstatus
    USING status::text::orderstatus;

DROP TYPE dborderstatus;
