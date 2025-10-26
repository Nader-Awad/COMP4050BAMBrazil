## PR Review Notes

- **Issues Identified**
  - Image capture flow stored files and metadata using IA-provided UUIDs, but `images.id` rows were created with new DB-generated IDs, so any follow-up API call using the capture response ID failed.
  - Image listing/search endpoints appended date filters to SQL even when date parsing failed, which caused placeholder/bind mismatches and 500s for malformed dates.
  - Authorization failures in image handlers returned HTTP 200 with `{"success":false}` payloads, making it impossible for clients/logging to distinguish forbidden access.

- **Fixes Implemented**
  - `DatabaseService::create_image` now inserts the IA UUID explicitly and returns the saved row, ensuring in-memory IDs match what is persisted.
  - Added a shared `parse_date_param` helper so handlers validate `date_from`/`date_to` early and pass typed `NaiveDate`s to the DB layer, which now only binds placeholders after a successful parse.
  - Updated all relevant image handlers to return real `403` responses (with warnings logged) whenever a user attempts to access records they do not own.

- **Additional Changes**
  - Introduced a `make dev` target plus `IA_MOCK_MODE` wiring in `docker-compose.yml` so the IA client can be mocked locally without impacting production defaults.
  - Documented the mock-mode workflow in the root README, backend README, and `.env.example` to ensure contributors know when/how to enable it.
