ALTER TABLE blog_posts ADD CONSTRAINT fk_author_id FOREIGN KEY (author_id) REFERENCES users(id);
