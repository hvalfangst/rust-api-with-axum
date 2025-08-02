-- Create the ships table
CREATE TABLE ships (
                      id SERIAL PRIMARY KEY,
                      name VARCHAR(100) NOT NULL,
                      category VARCHAR(50),
                      description TEXT,
                      empire_id INT REFERENCES empires(id) NOT NULL
);