CREATE TYPE dborderstatus AS ENUM (
    'New', 'InProgress', 'ReadyToShip', 'Shipping', 'Done'
);

ALTER TABLE orders
    ALTER COLUMN status
    TYPE dborderstatus
    USING status::text::dborderstatus;

DROP TYPE orderstatus;
