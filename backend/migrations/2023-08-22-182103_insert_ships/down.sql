-- Delete data from the 'ships' table
DELETE FROM ships
WHERE name IN (
               'Raven', 'Dominix', 'Catalyst', 'Moa', 'Cormorant',
               'Helios', 'Thorax', 'Vexor', 'Tristan', 'Celestis',
               'Apocalypse', 'Coercer', 'Impel', 'Malediction', 'Punisher',
               'Hurricane', 'Rifter', 'Typhoon', 'Stabber', 'Claw',
               'Succubus', 'Barghest', 'Daredevil', 'Worm', 'Marauder'
    );