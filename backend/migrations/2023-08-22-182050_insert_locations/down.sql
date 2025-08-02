-- Delete data from the 'locations' table
DELETE FROM locations
WHERE star_system IN (
                      'New Eden', 'Genesis', 'The Forge', 'Domain', 'Delve'
    );