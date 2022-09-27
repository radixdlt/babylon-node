Guidelines for logging in Rust

The goal of recording tracing information is to make the future debugging
easier.

Executive summary:
- use `tracing`
- don't worry about the performance overhead
- annotate any function of your choosing with the `tracing::instrument`
  attribute

It is unfeasible to prepare for any future debugging scenario (otherwise why
don't you prevent those bugs?), and these can be attached later as well, so do
not spend way too much effort on logging, just enough that makes your future
life easier.
Focus on creating small functions with descriptive name, and
attach the previously mentioned `instrument` attribute, which creates a `span`
with the name of the function. Also, the benefit/cost ratio of these
annotations are rather high, as they require little to no additional
maintenance.

See also:
- Guide to Rustc Development
  - Using tracing to debug the compiler
    https://rustc-dev-guide.rust-lang.org/tracing.html
