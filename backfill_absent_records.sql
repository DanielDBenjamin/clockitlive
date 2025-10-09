-- Backfill absent records for all completed classes that don't have full attendance
-- This script marks students as absent if they were enrolled but didn't attend

INSERT INTO attendance (studentID, classID, status, recorded_at, notes)
SELECT 
    u.userID, 
    c.classID, 
    'absent', 
    datetime('now'),
    'Backfilled absent record'
FROM classes c
INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
INNER JOIN users u ON u.emailAddress = ms.studentEmailAddress
WHERE c.status = 'completed'
  AND u.role = 'student'
  AND NOT EXISTS (
      SELECT 1 FROM attendance a 
      WHERE a.classID = c.classID 
      AND a.studentID = u.userID
  );

-- Show summary of what was added
SELECT 
    'Backfill Summary' as report,
    COUNT(*) as absent_records_added
FROM attendance 
WHERE notes = 'Backfilled absent record';
