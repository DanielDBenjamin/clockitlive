#!/usr/bin/env python3
"""
Generate SQL for 3 new modules for Lara Lecturer with classes and varied attendance
"""

import random
from datetime import datetime, timedelta

# Configuration
MODULES = [
    ('WEB301', 'Web Development & Design', 'Modern web technologies and responsive design'),
    ('NET401', 'Network Security', 'Cybersecurity principles and network protection'),
    ('SWE501', 'Software Engineering', 'Software development methodologies and best practices'),
]

LECTURER_EMAIL = 'lara.lecturer@example.edu'
TUTORS = ['tia.tutor@example.edu', 'tom.tutor@example.edu']
NUM_STUDENTS = 100
TIMESTAMP = '2025-10-07 09:36:16'

# Class schedule: Different days/times for each module
SCHEDULES = {
    'WEB301': {'days': 'TR', 'time': '10:00', 'venue': 'LT2', 'duration': 90},  # Tue/Thu
    'NET401': {'days': 'MW', 'time': '14:00', 'venue': 'LT3', 'duration': 90},  # Mon/Wed
    'SWE501': {'days': 'F', 'time': '11:00', 'venue': 'LT4', 'duration': 120},  # Friday
}

# Generate classes from Aug 4 to Oct 9, 2025
START_DATE = datetime(2025, 8, 4)
END_DATE = datetime(2025, 10, 9)

def get_weekday_abbrev(date):
    """Return day abbreviation: M, T, W, R (Thu), F"""
    days = ['M', 'T', 'W', 'R', 'F', '', '']  # Mon-Fri, skip Sat/Sun
    return days[date.weekday()]

def generate_classes_for_module(module_code, schedule):
    """Generate classes for a module based on schedule"""
    classes = []
    current = START_DATE
    
    while current <= END_DATE:
        day_abbrev = get_weekday_abbrev(current)
        if day_abbrev in schedule['days']:
            date_str = current.strftime('%Y-%m-%d')
            title = f"{module_code} Lecture {date_str}"
            
            # Mark classes in the future as 'upcoming', past as 'completed'
            status = 'upcoming' if current > datetime(2025, 10, 7) else 'completed'
            
            classes.append({
                'moduleCode': module_code,
                'title': title,
                'venue': schedule['venue'],
                'date': date_str,
                'time': schedule['time'],
                'status': status,
                'duration': schedule['duration'],
            })
        
        current += timedelta(days=1)
    
    return classes

def generate_attendance_for_class(class_id, module_code, student_emails):
    """Generate varied attendance records for a class"""
    attendance_records = []
    
    # Vary attendance rate by module to create diversity
    attendance_rates = {
        'WEB301': 0.82,  # 82% average attendance
        'NET401': 0.88,  # 88% average attendance
        'SWE501': 0.75,  # 75% average attendance
    }
    
    base_rate = attendance_rates.get(module_code, 0.80)
    
    # Add some weekly variation (±10%)
    week_variation = random.uniform(-0.10, 0.10)
    attendance_rate = max(0.60, min(0.95, base_rate + week_variation))
    
    for student_email in student_emails:
        # Each student has their own attendance tendency
        student_variation = random.uniform(-0.15, 0.15)
        student_rate = max(0.50, min(1.0, attendance_rate + student_variation))
        
        if random.random() < student_rate:
            # Student attended - generate location data
            lat = -33.932 + random.uniform(-0.001, 0.001)
            lon = 18.865 + random.uniform(-0.001, 0.001)
            accuracy = round(5.0 + random.random() * 15.0, 1)
            
            attendance_records.append({
                'student_email': student_email,
                'class_id': class_id,
                'lat': lat,
                'lon': lon,
                'accuracy': accuracy,
            })
    
    return attendance_records

def main():
    # Load actual student emails from existing migration
    student_emails = []
    with open('migrations/20251110000000_seed_dummy_data.sql', 'r') as f:
        for line in f:
            if "INSERT INTO users" in line and "student" in line:
                # Extract email from the line
                import re
                match = re.search(r"'([^']*@example\.edu)'", line)
                if match:
                    student_emails.append(match.group(1))
    
    print(f"Found {len(student_emails)} student emails")
    
    output = []
    output.append("-- Generated modules data for Lara Lecturer")
    output.append(f"-- Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    output.append("")
    
    # 1. Insert modules
    output.append("-- Insert new modules")
    for code, title, desc in MODULES:
        output.append(
            f"INSERT INTO modules (moduleCode,moduleTitle,description,created_at,updated_at) "
            f"VALUES ('{code}','{title}','{desc}','{TIMESTAMP}','{TIMESTAMP}');"
        )
    output.append("")
    
    # 2. Link lecturer to modules
    output.append("-- Link lecturer to modules")
    for code, _, _ in MODULES:
        output.append(
            f"INSERT INTO lecturer_module (moduleCode,lecturerEmailAddress,created_at) "
            f"VALUES ('{code}','{LECTURER_EMAIL}','{TIMESTAMP}');"
        )
    output.append("")
    
    # 3. Link tutors to modules (alternate between tutors)
    output.append("-- Link tutors to modules")
    for idx, (code, _, _) in enumerate(MODULES):
        tutor = TUTORS[idx % len(TUTORS)]
        output.append(
            f"INSERT INTO module_tutor (moduleCode,tutorEmailAddress,created_at) "
            f"VALUES ('{code}','{tutor}','{TIMESTAMP}');"
        )
    output.append("")
    
    # 4. Enroll all students in all new modules
    output.append("-- Enroll all students in new modules")
    for code, _, _ in MODULES:
        for email in student_emails:
            output.append(
                f"INSERT INTO module_students (moduleCode,studentEmailAddress,created_at) "
                f"VALUES ('{code}','{email}','{TIMESTAMP}');"
            )
    output.append("")
    
    # 5. Generate classes for each module
    output.append("-- Generate classes for each module")
    all_classes = {}
    class_counter = 1000  # Start from high number to avoid conflicts
    
    for code, _, _ in MODULES:
        schedule = SCHEDULES[code]
        classes = generate_classes_for_module(code, schedule)
        all_classes[code] = []
        
        for cls in classes:
            class_id = class_counter
            class_counter += 1
            
            output.append(
                f"INSERT INTO classes (moduleCode,title,venue,description,recurring,date,time,status,"
                f"created_at,updated_at,duration_minutes,created_by) "
                f"VALUES ('{cls['moduleCode']}','{cls['title']}','{cls['venue']}','Weekly session',"
                f"'{schedule['days']}','{cls['date']}','{cls['time']}','{cls['status']}',"
                f"'{TIMESTAMP}','{TIMESTAMP}',{cls['duration']},'{LECTURER_EMAIL}');"
            )
            
            # Store for attendance generation
            if cls['status'] == 'completed':
                all_classes[code].append({
                    'id': class_id,
                    'date': cls['date'],
                    'time': cls['time'],
                })
    
    output.append("")
    
    # 6. Generate class sessions and attendance
    output.append("-- Generate class sessions for completed classes")
    for code in MODULES:
        module_code = code[0]
        for cls in all_classes.get(module_code, []):
            # Parse date and time
            class_dt = datetime.strptime(f"{cls['date']} {cls['time']}", '%Y-%m-%d %H:%M')
            started_at = (class_dt - timedelta(minutes=5)).strftime('%Y-%m-%d %H:%M:%S')
            ended_at = (class_dt + timedelta(minutes=SCHEDULES[module_code]['duration'])).strftime('%Y-%m-%d %H:%M:%S')
            
            output.append(
                f"INSERT INTO class_sessions (classID, started_at, ended_at, started_by, "
                f"start_latitude, start_longitude, start_accuracy, location_radius)"
            )
            output.append(
                f"SELECT c.classID, '{started_at}', '{ended_at}', '{LECTURER_EMAIL}', "
                f"-33.932, 18.865, 15.0, 30.0"
            )
            output.append(
                f"FROM classes c WHERE c.moduleCode='{module_code}' AND c.date='{cls['date']}' "
                f"AND c.time='{cls['time']}';"
            )
    
    output.append("")
    output.append("-- Generate attendance records with varied patterns")
    
    for code in MODULES:
        module_code = code[0]
        for cls in all_classes.get(module_code, []):
            # Generate attendance for this class
            attendance = generate_attendance_for_class(0, module_code, student_emails)
            
            for att in attendance:
                output.append(
                    f"INSERT INTO attendance (studentID, classID, status, recorded_at, "
                    f"check_latitude, check_longitude, location_accuracy)"
                )
                output.append(
                    f"SELECT u.userID, c.classID, 'present', "
                    f"datetime(c.date || ' ' || c.time, '+{random.randint(-5, 5)} minutes'), "
                    f"{att['lat']:.6f}, {att['lon']:.6f}, {att['accuracy']}"
                )
                output.append(
                    f"FROM users u, classes c "
                    f"WHERE u.emailAddress='{att['student_email']}' "
                    f"AND c.moduleCode='{module_code}' AND c.date='{cls['date']}' AND c.time='{cls['time']}';"
                )
    
    # Write to file
    output_file = 'migrations/20251111000000_add_three_modules.sql'
    with open(output_file, 'w') as f:
        f.write('\n'.join(output))
    
    print(f"✅ Generated {output_file}")
    print(f"   📚 Modules: {len(MODULES)}")
    print(f"   🎓 Classes per module: ~{len(generate_classes_for_module(MODULES[0][0], SCHEDULES[MODULES[0][0]]))} each")
    print(f"   👥 Students enrolled: {NUM_STUDENTS} per module")
    print(f"   📊 Attendance: Varied (60-95% with weekly/student variations)")

if __name__ == '__main__':
    main()
