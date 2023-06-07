[![codecov](https://codecov.io/gh/simlay/line-server-exercise/branch/main/graph/badge.svg?token=J4Z0HON7UV)](https://codecov.io/gh/simlay/line-server-exercise)
[![Security Audit](https://github.com/simlay/line-server-exercise/actions/workflows/security-audit.yml/badge.svg)](https://github.com/simlay/line-server-exercise/actions/workflows/security-audit.yml)
[![CI](https://github.com/simlay/line-server-exercise/actions/workflows/ci.yml/badge.svg)](https://github.com/simlay/line-server-exercise/actions/workflows/ci.yml)

# Intro

See [PROPMT.md](./PROMPT.md) for the exact description of this exercise.

# Additions

* The Specification said that any error should return `ERR\r\n`. I felt that
this was not descriptive enough and so the responses for Errors are more human
readable.
    - In the event of an known command, `Err - THIS_COMMAND_DOES_NOT_EXIST is an invalid command. \`GET nnnn | QUIT | SHUTDOWN\` are valid commands.\r\n`
    - In the event that the `GET nnnn` command is out of bounds for the number of lines in the input text, something like `Err - failed to retrieve line 1000. There are only 4 lines available.\r\n`
    - In the event that the `GET nnnn` has an unparsible `usize` digit or is not a digit something like `Err - invalid digit found in string. Is AOEU an unsigned integer or under usize::MAX?\r\n` will be returned.

# Usage

In one terminal run:
```bash
cargo run -- --line-file ./example.txt
```

In another terminal run:
```bash
socat STDIO TCP4:localhost:10497
```

In the 2nd terminal type on of:
* `GET 1`
* `QUIT`
* `SHUTDOWN`

# Questions and Answers
Q: How does your system work? (if not addressed in comments in source)
* This system reads in a file line by line, then shares that `Vec<String>`
across multiple tokio green threeds for each local connection

Q: How will your system perform as the number of requests per second increases?
* Right now this is system uses the default tokio thread setup. This should
scale well with new clients and requests per second.

Q: How will your system perform with a 1 GB file? a 100 GB file? a 1,000 GB file?
* This system was not designed to optimize for a reduced memory footprint. It
was designed to handle many clients simultaneously.

Q: What documentation, websites, papers, etc did you consult in doing this assignment?
* [`tokio::select`](https://tokio.rs/tokio/tutorial/select) and other tokio objects.

Q: What third-party libraries or other tools does the system use?
* [tokio](https://crates.io/crates/tokio)
* [log](https://crates.io/crates/log)
* [env_logger](https://crates.io/crates/env_logger)
* [anyhow](https://crates.io/crates/anyhow)
* [clap](https://crates.io/crates/clap)
* [pretty_assertions](https://crates.io/crates/pretty_assertions)

Q: How long did you spend on this exercise?
* ~3 hours.


# Future work

Given that this is purely an exercise, I elected not to build out too many
features. I played around with using
[criterion](https://bheisler.github.io/criterion.rs/book/getting_started.html)
to use of `cargo bench` but found the tooling a bit more complicated than I
wanted for this project.

This application looks to just be using one core. There is a
`one_hundred_clients` test where there are 100 clients requesting 400 lines of
a `40000` line file with little addition to CI runtime.
