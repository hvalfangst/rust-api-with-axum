-- Create the players table
CREATE TABLE players (
                        id SERIAL PRIMARY KEY,
                        user_id INT REFERENCES users(id) NOT NULL,
                        active_ship_id INT REFERENCES ships(id) NOT NULL,
                        location_id INT REFERENCES locations(id) NOT NULL
);