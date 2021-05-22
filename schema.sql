CREATE TABLE candidates(
    id INT UNSIGNED PRIMARY KEY,
    name VARCHAR(255)
);

CREATE TABLE votes(
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user BIGINT UNSIGNED,
    option INT UNSIGNED,
    choice_number INT UNSIGNED,
    CONSTRAINT fk_option FOREIGN KEY (option) REFERENCES candidates(id)
);

CREATE INDEX choice_number_index ON votes(choice_number);
CREATE INDEX user_index ON votes(user);