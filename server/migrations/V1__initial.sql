CREATE EXTENSION hll;

CREATE TABLE application(
    id UUID PRIMARY KEY,
    name TEXT UNIQUE
);

CREATE TABLE cpu_manufacturer(
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE os(
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE cpu_capabilities(
    -- Time, truncated to day in utc.
    day TIMESTAMP WITH TIME ZONE NOT NULL,
    os UUID NOT NULL,
    cpu_manufacturer UUID NOT NULL,
    application UUID NOT NULL,

    x86_sse2 BOOLEAN NOT NULL,
    x86_sse3 BOOLEAN NOT NULL,
    x86_ssse3 BOOLEAN NOT NULL,
    x86_sse4_1 BOOLEAN NOT NULL,
    x86_avx BOOLEAN NOT NULL,
    x86_avx2 BOOLEAN NOT NULL,
    x86_fma3 BOOLEAN NOT NULL,
    x86_avx512f BOOLEAN NOT NULL,

    users_by_id hll,
    users_by_ip hll,

    FOREIGN KEY(os) REFERENCES os(id),
    FOREIGN KEY(cpu_manufacturer) REFERENCES cpu_manufacturer(id),
    FOREIGN KEY (application) REFERENCES application(id)
);

CREATE INDEX cpu_capabilities_day ON cpu_capabilities(day);
CREATE INDEX cpu_capabilities_os ON cpu_capabilities(os);
CREATE INDEX cpu_capabilities_application ON cpu_capabilities(application);

-- This table tracks the country as reported from Cloudflare.  See
-- https://developers.cloudflare.com/fundamentals/get-started/reference/http-request-headers/
CREATE TABLE cf_country(
    application UUID REFERENCES application(id) NOT NULL,
    day TIMESTAMP WITH TIME ZONE NOT NULL,
    country TEXT NOT NULL,

    users_by_id hll NOT NULL,
    users_by_ip hll NOT NULL
);

CREATE INDEX cf_country_day ON cf_country(day);
CREATE INDEX cf_country_country ON cf_country(country);

-- "hwsurvey_voyager" with this known uuid is our debug application.
INSERT INTO application(id, name) VALUES
    ('09bc8ff8-e452-11ec-be06-00d8612ce6ed', 'hwsurvey_voyager');

INSERT INTO cpu_manufacturer(id, name) VALUES
    ('7741323e-e454-11ec-8fa5-00d8612ce6ed', 'intel'),
    ('7b447b70-e454-11ec-b720-00d8612ce6ed', 'apple');

INSERT INTO os(id, name) VALUES
    ('81f9efea-e454-11ec-be9f-00d8612ce6ed', 'windows'),
    ('8552a506-e454-11ec-a0c3-00d8612ce6ed', 'linux'),
    ('88f282bc-e454-11ec-b882-00d8612ce6ed', 'macos');
