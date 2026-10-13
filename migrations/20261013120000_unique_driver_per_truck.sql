CREATE UNIQUE INDEX IF NOT EXISTS trucks_driver_id_key ON trucks (driver_id) WHERE driver_id IS NOT NULL;
