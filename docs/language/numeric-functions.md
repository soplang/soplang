# Numeric functions and seeded random streams

The 3.2 numeric-library stage adds logarithms, trigonometry, named constants,
clamping, and context-local reproducible random generation. These builtins work
through both Cranelift JIT and LLVM AOT and share the existing numeric error
code `E044`.

## Logarithms

`loog(value)` returns the natural logarithm. `loog(value, base)` returns the
logarithm in the given base. The value must be positive; the base must be finite,
positive, and not equal to `1`.

## Trigonometry

`sin`, `cos`, and `tan` evaluate in radians. Pass `"rogan"` as the second
argument for the inverse function. Inverse sine and cosine require inputs in
`[-1, 1]`.

`xagal(y, x)` returns the angle in radians from the origin to `(x, y)`, using
the same quadrant rules as `atan2`. Signed zero is preserved on either axis.

## Constants

`joogto(name)` returns a named floating-point constant. Supported names are
`"pi"`, `"e"`, `"tau"`, `"inf"`, and `"nan"`.

## Clamping

`xadid(value, low, high)` returns `value` bounded to `[low, high]`. When all
arguments are integers, the result stays an integer. Mixed or floating inputs
follow the existing numeric comparison rules.

## Seeded random streams

`abuur(seed)` resets the context-local random stream. It accepts a single
integer seed and returns `maran`. Each runtime context owns its own stream;
nested activation and collection do not leak state across contexts.

`xul()` draws a float in `[0, 1)`. `xul(low, high)` draws a float in
`[low, high)` with a finite upper endpoint excluded. `xul(list)` returns one
element from a non-empty list. Invalid calls, including conflicting list
borrows, do not advance the stream.

When no seed was set, the first draw in a context uses a one-time stream derived
from the clock. Re-seeding with the same integer reproduces the same sequence of
draws and list choices.

## Integer extrema

`yar(...)` and `weyn(...)` return the smallest or largest argument. When every
argument is an integer, the result keeps full 64-bit precision. `ugu_yar` and
`ugu_weyn` delegate to the same implementation.

## Errors

Domain and type failures report Somali messages with `[E044]`. Examples include
non-positive logarithm inputs, inverse trigonometry outside `[-1, 1]`, invalid
constant names, inverted clamp bounds, non-integer seeds, empty list choice,
and non-finite random bounds.
