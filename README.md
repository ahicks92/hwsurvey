# hwsurvey

## What?

I needed to do the Steam hardware survey but specifically of blind people which tend to have weirder hardware, and
specifically for a subset of info (plus a few things they don't have).  This glues together some code from myself for
cpuid parsing and a few Rust crates into something which can collect info via a drop-in client library/binary in a
fashion that is pretty much anonymous and spit it at Postgres.

For my personal use this is going to factor into [synthizer](https://github.com/synthizer/synthizer) development by
revealing which SIMD extensions to target, and what some common cache sizes are in what is in effect my primary audience
there as of this writing.  I'm also going to be using it to figure out which locales/languages people are in.

It seemed useful to others plus open sourcing the thing you're collecting supposedly anonymous data with is good in all
the ways.  I may or may not open an instance of this up more widely and permanently (that said, if you want one,
`docker compose up` works--but you'll need to do some tweaking to make that persistent.  I'm not providing instructions
for now).

## What do we collect?

We collect the following (see below w.r.t. anonymization and preventing fingerprinting):

- Os, currently limited to "kind", e.g. windows, but not windows 10.
- CPU architecture: aarch, x86, etc.
- CPU features: sse/sse2/avx/etc.
- CPU cache sizes of l1, l2, and l3.
- Total memory.
- Cloudflare-reported country and ip.

## Anonymization

NOTE: during the initial volunteer/tester rollout, we will keep shipped payloads in order to build a test corpus and
make sure the world is sane; this will quickly be removed from the codebase.

If you have the user's exact CPU, memory, cache sizes, ip, etc etc. it becomes trivial to fingerprint.  We want to
prevent that.  But we also need to be able to not double count machines.  We can't be perfect, but we can limit what we
do have in ways that make this harder.

I currently do not believe it is possible to use this software to connect a user
to a datapoint after the fact without access to their physical machine at the same time, and am reasonably convinced it
isn't possible to definitively do so even with it.  Feel free to correct me.

In practice, CPU flags only narrow down to a generation, not a specific model.  OS without version doesn't really tell
you much either.  We don't collect frequency, so the best we get is "maybe an i5 from before avx running Windows".  That
only leaves CPU caches, geolocation, and machine ids.

We are also careful in the case of OS to only collect OSes we currently care about, and bucket everything else as
"unknown".

I consider the usefulness of geolocation to outweigh the lack of anonymity, and it's only down to two-letter country
codes.  See [here](https://developers.cloudflare.com/fundamentals/get-started/reference/http-request-headers/). This
information is also already leaked by simply downloading the software, visiting web sites, etc.  Put another way: no one
needs what I wrote here to collect it, in fact it's often on in everything web server related and you have to go turn it
off.

The risk with CPU caches and memory is that they do start narrowin down the CPU/machine model quite a lot.  Fortunately
mostly what we need to know is "how many people are over 4GB?"  To anonymize this data, we divide these into bands.  For
example 4.2, 4.6, 4.8, etc. all get recorded as 4.  This is hard-coded (see anonymization.rs for the lists) and applied
to the data before storage.  Note that this happens server-side: if we did it client side it would not be possible to
change the bands while still getting useful info from client software that may slowly or never update.

This brings us to machine ids and IPs.  We use [hyperloglog](https://en.wikipedia.org/wiki/HyperLogLog) to store an
approximate count without storing the values.  It is not possible to reverse a hyperloglog; to see this, consider that
it is `O(1)` on memory.  We maintain one hyperloglog of machine ids and a separate, unrelated hyperloglog of IPs.

The machine id itself is the machine's first MAC address and hostname, double-hashed with sha256, and is transmitted
over the wire in this form.  The reason we also take IP is that this is clearly manipulable and not accurate; the hope
is that having both will allow finding more signal in the potential noise.

Finally, we don't store tables in a joinable fashion, and we truncate timestamps to day.  This means that the most this
service can do is build pre-defined reports with day granularity over pre-joined tables.  By design, it's not possible
to (for example) join the CPU capabilities table and the memory table to ask how many people with AVX are over 4 GB, or
to ask how many people in England have an l2 cache of more than 8kb.

the weakness in this scheme is that the payload does still need to be transmitted.  If there is a MITM the extra
measures the backend takes aren't applied.  This is still way less than, e.g., Sentry collects with respect to being
able to identify people.

## Integration

TBD, basically you need to ask me and I need to know who you are.

## Reports

there will be nice HTML public reports eventually.
