INSERT INTO districts (name, collection_day) VALUES
    ('Ikeja', 'Monday'),
    ('Surulere', 'Tuesday'),
    ('Eti-Osa', 'Wednesday');

INSERT INTO users (full_name, email, role) VALUES
    ('Ngozi Eze',      'ngozi@example.com',   'supervisor'),
    ('Chinedu Okafor', 'chinedu@example.com', 'driver'),
    ('Aisha Bello',    'aisha@example.com',   'driver');

INSERT INTO trucks (fleet_code, driver_id, supervisor_id, shift_start, shift_end)
SELECT 'TT-0023', d.id, s.id, 6, 14
FROM users d, users s
WHERE d.email = 'chinedu@example.com' AND s.email = 'ngozi@example.com';

INSERT INTO trucks (fleet_code, driver_id, supervisor_id, shift_start, shift_end)
SELECT 'TT-0034', d.id, s.id, 6, 14
FROM users d, users s
WHERE d.email = 'aisha@example.com' AND s.email = 'ngozi@example.com';

INSERT INTO routes (truck_id, district_id, street, lane, first_house, last_house)
SELECT t.id, d.id, 'Allen Avenue', 2, 1, 50
FROM trucks t, districts d WHERE t.fleet_code = 'TT-0023' AND d.name = 'Ikeja';

INSERT INTO routes (truck_id, district_id, street, lane, first_house, last_house)
SELECT t.id, d.id, 'Allen Avenue', 2, 51, 100
FROM trucks t, districts d WHERE t.fleet_code = 'TT-0034' AND d.name = 'Ikeja';