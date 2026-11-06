# Preview compatibility

| Area | 0.1.0a1 scope |
|---|---|
| Python | CPython 3.12 x64; synchronous native calls; fresh session per render |
| Platforms | Windows x64; Linux x64 candidate requiring glibc 2.35 or newer, initially Ubuntu 22.04 |
| Scalars | Strict dotted targets; strings, finite numbers, booleans; one replacement style |
| Rows | Complete top-level data-row prototype; documented merge/reference restrictions |
| Pictures | Uncropped inline raster slots; aspect-ratio fit; explicit optional absence |
| Blocks | Complete top-level repeat and boolean conditional blocks; scoped rows/pictures |
| Diagnostics | Shared inspection/preflight/render target resolution; actionable TemplateError |
| Output | New staging paths only; failed writes may leave partial staging files |
| Compatibility | Tested fixture scope only; new templates need structure and Word review |
| Not supported | Desktop release, nested variable blocks, arbitrary expressions, floating/cropped picture slots, reference-bearing repeat prototypes |

Preflight does not prove filesystem success or visual fidelity. Runtime/font
requirements and separate-machine installation remain acceptance checks.
See the binding README for exact API contracts and limits.
