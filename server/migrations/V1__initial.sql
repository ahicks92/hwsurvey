CREATE EXTENSION hll;

CREATE TABLE cpu_manufacturer(
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE os(
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE cpu_capabilities(
    -- Time, truncated to day in utc.
    day TIMESTAMP WITH TIME ZONE NOT NULL,
    os INTEGER NOT NULL,
    cpu_manufacturer INTEGER NOT NULL,

    x86_sse2 BOOLEAN NOT NULL,
    x86_sse3 BOOLEAN NOT NULL,
    x86_ssse3 BOOLEAN NOT NULL,
    x86_sse4_1 BOOLEAN NOT NULL,
    x86_avx BOOLEAN NOT NULL,
    x86_avx2 BOOLEAN NOT NULL,
    x86_fma3 BOOLEAN NOT NULL,
    x86_avx BOOLEAN NOT NULL,
    x86_avx2 BOOLEAN NOT NULL,
    x86_avx512f BOOLEAN NOT NULL,

    users hll,

    FOREIGN KEY(os) REFERENCES os(id),
    FOREIGN KEY(cpu_manufacturer) REFERENCES cpu_manufacturer(id)
);


INSERT INTO cpu_manufacturer(id, name) VALUES
    (1, "intel"),
    (2, "apple");

INSERT INTO os(id, name) VALUES
    (1, "windows"),
    (2, "linux"),
    (3, "macos");
