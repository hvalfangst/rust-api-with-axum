-- Delete data from the 'empires' table
DELETE FROM empires
WHERE name IN (
               'Caldari State', 'Gallente Federation', 'Amarr Empire', 'Minmatar Republic', 'Pirate Coalition'
    );