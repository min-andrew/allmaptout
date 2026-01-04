CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('ceremony', 'reception', 'other')),
    event_date DATE NOT NULL,
    event_time TIME NOT NULL,
    location_name TEXT NOT NULL,
    location_address TEXT NOT NULL,
    description TEXT,
    display_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX events_display_order_idx ON events(display_order);
