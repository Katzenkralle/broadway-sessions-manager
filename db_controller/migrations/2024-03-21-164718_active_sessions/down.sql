-- This file should undo anything in `up.sql`
DROP TABLE active_sessions;

CREATE TABLE "user_service_map" (
    "username" VARCHAR(255),
    "service_id" INTEGER,
    "data_location" TEXT,
    FOREIGN KEY ("username") REFERENCES "users"("username") ON DELETE CASCADE,
    FOREIGN KEY ("service_id") REFERENCES "services"("id") ON DELETE CASCADE,
    PRIMARY KEY ("username","service_id")
);