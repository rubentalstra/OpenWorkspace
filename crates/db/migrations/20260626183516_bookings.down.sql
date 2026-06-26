-- Reverse of bookings.up.sql, in reverse dependency order.
DROP TABLE IF EXISTS booking_occurrences;
DROP TABLE IF EXISTS booking_recurrence_overrides;
DROP TABLE IF EXISTS bookings;
DROP FUNCTION IF EXISTS bookings_validate_series_timezone();
