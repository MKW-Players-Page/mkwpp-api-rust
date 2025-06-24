CREATE TABLE blog_posts (
    id SERIAL NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    is_published BOOLEAN NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL,
    author_id INTEGER NOT NULL
);