
DROP TRIGGER IF EXISTS set_updated_at ON Student;
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP TABLE IF EXISTS Student;

CREATE TABLE IF NOT EXISTS Student (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    birthdate DATE NOT NULL,
    score INT CHECK (score >= 0 AND score <= 100),
    data JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create a trigger function to update the updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach the trigger to the Student table
CREATE TRIGGER set_updated_at
BEFORE UPDATE ON Student
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

INSERT INTO Student (name, birthdate, email, score)
    VALUES ('Foo', '05-05-1989', 'foo@mail.com', 10);