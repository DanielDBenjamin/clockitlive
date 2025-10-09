-- Add created_by field to track who created each class
ALTER TABLE classes ADD COLUMN created_by TEXT;

-- Set default value for existing classes to be created by lecturers
-- We'll update this to be more specific in the application code
UPDATE classes SET created_by = 'lecturer' WHERE created_by IS NULL;