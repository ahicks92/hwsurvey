CREATE EXTENSION hll;

CREATE TABLE application(
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE cpu_manufacturer(
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE os(
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE cpu_architecture(
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE cpu_capabilities(
    -- Time, truncated to day in utc.
    day TIMESTAMP WITH TIME ZONE NOT NULL,
    application UUID NOT NULL,
    os UUID NOT NULL,
    cpu_manufacturer UUID NOT NULL,
    architecture UUID NOT NULL,

    x86_sse2 BOOLEAN NOT NULL,
    x86_sse3 BOOLEAN NOT NULL,
    x86_ssse3 BOOLEAN NOT NULL,
    x86_sse4_1 BOOLEAN NOT NULL,
    x86_fma3 BOOLEAN NOT NULL,
    x86_avx BOOLEAN NOT NULL,
    x86_avx2 BOOLEAN NOT NULL,
    x86_avx512f BOOLEAN NOT NULL,

    users_by_id hll NOT NULL,
    users_by_ip hll NOT NULL,

    FOREIGN KEY(os) REFERENCES os(id),
    FOREIGN KEY(cpu_manufacturer) REFERENCES cpu_manufacturer(id),
    FOREIGN KEY (application) REFERENCES application(id),
    FOREIGN KEY (architecture) REFERENCES cpu_architecture(id),

    -- Enables insert or update to insert into the hlls.
    CONSTRAINT cpu_capabilities_upsert_constraint UNIQUE(day, application, cpu_manufacturer, os, architecture, x86_sse2, x86_sse3, x86_ssse3, x86_sse4_1,
        x86_avx, x86_avx2, x86_avx512f)
);

CREATE INDEX cpu_capabilities_day ON cpu_capabilities(day);
CREATE INDEX cpu_capabilities_os ON cpu_capabilities(os);
CREATE INDEX cpu_capabilities_application ON cpu_capabilities(application);
CREATE INDEX cpu_capabilities_architecture ON cpu_capabilities(architecture);

CREATE TABLE cpu_caches(
        day TIMESTAMP WITH TIME ZONE NOT NULL,
    application UUID REFERENCES application(id) NOT NULL,

    l1i BIGINT NOT NULL,
    l1d BIGINT NOT NULL,
    l1u BIGINT NOT NULL,
    l2i BIGINT NOT NULL,
    l2d BIGINT NOT NULL,
    l2u BIGINT NOT NULL,
    l3i BIGINT NOT NULL,
    l3d BIGINT NOT NULL,
    l3u BIGINT NOT NULL,

    users_by_id hll,
    users_by_ip hll,

    CONSTRAINT cpu_caches_upsert_constraint UNIQUE(application, day, l1i, l1d, l1u, l2i, l2d, l2u, l3i, l3d, l3u)
);

CREATE INDEX cpu_caches_day ON cpu_caches(day);

CREATE TABLE memory(
    application UUID NOT NULL REFERENCES application(id),
    day TIMESTAMP WITH TIME ZONE NOT NULL,
    total_memory BIGINT NOT NULL,

    users_by_id hll,
    users_by_ip hll,

    CONSTRAINT memory_upsert_constraint UNIQUE(application, day, total_memory)
);

CREATE INDEX memory_application ON memory(application);
CREATE INDEX memory_day ON memory(day);

-- This table tracks the country as reported from Cloudflare.  See
-- https://developers.cloudflare.com/fundamentals/get-started/reference/http-request-headers/
CREATE TABLE cf_country(
    application UUID REFERENCES application(id) NOT NULL,
    day TIMESTAMP WITH TIME ZONE NOT NULL,
    country TEXT NOT NULL,

    users_by_id hll NOT NULL,
    users_by_ip hll NOT NULL,

    CONSTRAINT cf_country_upsert_constraint UNIQUE(application, day, country)
);

CREATE INDEX cf_country_day ON cf_country(day);
CREATE INDEX cf_country_country ON cf_country(country);

-- "hwsurvey_voyager" with this known uuid is our debug application.
INSERT INTO application(id, name) VALUES
    ('09bc8ff8-e452-11ec-be06-00d8612ce6ed', 'hwsurvey_voyager');

INSERT INTO cpu_manufacturer(id, name) VALUES
    -- Unknown is a special name; we use it when we don't have anything else.
    ('8f972a0c-e68c-11ec-b046-00d8612ce6ed', 'unknown'),
    ('7741323e-e454-11ec-8fa5-00d8612ce6ed', 'intel'),
    ('7b447b70-e454-11ec-b720-00d8612ce6ed', 'apple');

INSERT INTO os(id, name) VALUES
    ('832060f4-e68c-11ec-9a85-00d8612ce6ed', 'unknown'),
    ('81f9efea-e454-11ec-be9f-00d8612ce6ed', 'windows'),
    ('8552a506-e454-11ec-a0c3-00d8612ce6ed', 'linux'),
    ('88f282bc-e454-11ec-b882-00d8612ce6ed', 'macos');

INSERT into cpu_architecture(id, name) VALUES
    ('24e7984e-e68d-11ec-8e48-00d8612ce6ed', 'unknown'),
    ('761f46a4-e457-11ec-9b5d-00d8612ce6ed', 'aarch64'),
    ('90131126-e457-11ec-9405-00d8612ce6ed', 'x86');
