CREATE TABLE company_settings (
 id INTEGER PRIMARY KEY CHECK(id=1),
 record_json TEXT NOT NULL CHECK(json_valid(record_json))
);
