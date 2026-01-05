-- Update event_type constraint to support all event types from the frontend
ALTER TABLE events DROP CONSTRAINT events_event_type_check;

ALTER TABLE events ADD CONSTRAINT events_event_type_check
    CHECK (event_type IN ('ceremony', 'reception', 'rehearsal', 'welcome', 'brunch', 'other'));
