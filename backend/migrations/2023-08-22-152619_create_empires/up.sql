-- Create the empires table
CREATE TABLE empires (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR(100) NOT NULL,
                        slogan VARCHAR(100) NOT NULL,
                        location_id INT REFERENCES locations(id) NOT NULL,
                        description TEXT NOT NULL
);