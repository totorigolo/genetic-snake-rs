# genetic-snake-rust

Teaching a bot to play Snake, without telling it anything
about the rules, using a genetic algorithm.

The bot that the genetic algorithm tries to train is based
on heuristic values: at each step it computes a set of
values for each move. Then, it computes a weighted sum for
each action and executes the one with the highest value.

I choose "good weights" as default values for the
`HeuristicBot`, which serves as a reference. Then, the
genetic algorithm tries to learn weights which beat my
good weights.

To see/tune the genetic algorithm parameters and methods,
go to `src/learning.rs`.

To see what are the heuristic values and how they are
computed, go to `src/heuristic_bot.rs`.

[![asciicast](https://asciinema.org/a/241704.svg)](https://asciinema.org/a/241704)

# Usage

To run `genetic-snake-rust`, a recent stable Rust should work.

The easiest way to get Rust is via
[`rustup`](https://www.rust-lang.org/en-US/install.html).

Then, simply run:

```
cargo run --release
```

# License & Contributing

This repository is licensed under the permissive MIT
license, meaning that you can and are encouraged to hack
this code!

Any PR is welcomed! :snake:
