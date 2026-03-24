-- this file is for documentation / reference only

-- users table
CREATE TABLE IF NOT EXISTS users (
    user_id       SERIAL PRIMARY KEY,
    reg_number    VARCHAR(50)  UNIQUE NOT NULL,
    name          VARCHAR(100) NOT NULL,
    email         VARCHAR(150) UNIQUE NOT NULL,
    password      VARCHAR(255) NOT NULL,
    vjudge_handle    VARCHAR(100),
    codeforces_handle VARCHAR(100),
    is_admin      BOOLEAN DEFAULT false,
    status        VARCHAR(20) DEFAULT 'pending'
);

-- contests table
CREATE TABLE IF NOT EXISTS contests (
    contest_no   SERIAL PRIMARY KEY,
    title        VARCHAR(255) NOT NULL,
    contest_link VARCHAR(255) NOT NULL,
    contest_date TIMESTAMP,
    created_at   TIMESTAMP DEFAULT NOW()
);

-- announcements table
CREATE TABLE IF NOT EXISTS announcements (
    post_id    SERIAL PRIMARY KEY,
    author_id  INTEGER REFERENCES users(user_id),
    title      VARCHAR(255) NOT NULL,
    content    TEXT NOT NULL,
    category   VARCHAR(50),
    event_date TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
