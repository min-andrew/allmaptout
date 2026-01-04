CREATE TABLE invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT UNIQUE NOT NULL,
    code_type TEXT NOT NULL CHECK (code_type IN ('guest', 'admin')),
    guest_id UUID REFERENCES guests(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX invite_codes_code_idx ON invite_codes(code);
