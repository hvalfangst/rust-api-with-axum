-- Create the locations table
CREATE TABLE locations (
                          id SERIAL PRIMARY KEY,
                          star_system VARCHAR(100) NOT NULL,
                          area VARCHAR(100) NOT NULL
);