-- Your SQL goes here
CREATE TABLE "users" (
    "username" VARCHAR(255) PRIMARY KEY NOT NULL,
    "password" VARCHAR(255),
    "role" VARCHAR(255) NOT NULL
    );

CREATE TABLE "services" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    "name" VARCHAR(255) NOT NULL,
    "description" TEXT
);

CREATE TABLE "user_service_map" (
    "username" VARCHAR(255),
    "service_id" INTEGER,
    "data_location" TEXT,
    FOREIGN KEY ("username") REFERENCES "users"("username") ON DELETE CASCADE,
    FOREIGN KEY ("service_id") REFERENCES "services"("id") ON DELETE CASCADE,
    PRIMARY KEY ("username","service_id")
);
