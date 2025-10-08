-- Add university field back to users table
ALTER TABLE users ADD COLUMN university TEXT NOT NULL DEFAULT 'Stellenbosch University';
