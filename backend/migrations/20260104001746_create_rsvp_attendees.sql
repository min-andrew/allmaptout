CREATE TABLE rsvp_attendees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rsvp_id UUID NOT NULL REFERENCES rsvps(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    is_attending BOOLEAN NOT NULL,
    meal_preference TEXT CHECK (meal_preference IN ('beef', 'chicken', 'fish', 'vegetarian', 'vegan')),
    dietary_restrictions TEXT,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX rsvp_attendees_rsvp_id_idx ON rsvp_attendees(rsvp_id);
