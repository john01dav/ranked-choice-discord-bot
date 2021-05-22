CREATE TABLE candidates(
    id INT UNSIGNED PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE votes(
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY NOT NULL,
    user BIGINT UNSIGNED NOT NULL,
    option INT UNSIGNED NOT NULL,
    choice_number INT UNSIGNED NOT NULL,
    CONSTRAINT fk_option FOREIGN KEY (option) REFERENCES candidates(id)
);

CREATE INDEX choice_number_index ON votes(user, choice_number);
CREATE INDEX user_index ON votes(user);