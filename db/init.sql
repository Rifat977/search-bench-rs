CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT,
    brand TEXT,
    description TEXT,
    price REAL,
    currency TEXT,
    availability TEXT,
    reviews_count INTEGER,
    rating REAL,
    discount INTEGER,
    manufacturer TEXT,
    category TEXT
);
