# `counts`

`counts` tallies line frequencies within text files, like an improved version
of the Unix command chain `sort | uniq -c`.

You can use it in combination with logging print statements in a program of
interest to obtain invaluable, domain-specific profiling data. I call this "ad
hoc profiling".

# Building

Within the `counts` repository, run `cargo build --release`. This creates the
binary `target/release/counts`. You can then put that directory in your `PATH`
variable, or copy/symlink the binary elsewhere.

# An example

Consider the following input.
```
a 1
b 2
c 3
d 4
d 4
c 3
c 3
d 4
b 2
d 4
```
`counts` produces the following output.
```
10 counts:
(  1)        4 (40.0%, 40.0%): d 4
(  2)        3 (30.0%, 70.0%): c 3
(  3)        2 (20.0%, 90.0%): b 2
(  4)        1 (10.0%,100.0%): a 1
```
It gives a total line count, and shows all the unique lines, ordered by
frequency, with individual and cumulative percentages.

Alternatively, when invoked with the `-w` flag, it assigns each line a weight,
determined by the last integer that appears on the line (or 1 if there is no
such integer).  On the same input, `counts -w` produces the following output.
```
30 counts:
(  1)       16 (53.3%, 53.3%): d 4
(  2)        9 (30.0%, 83.3%): c 3
(  3)        4 (13.3%, 96.7%): b 2
(  4)        1 ( 3.3%,100.0%): a 1
```
The total and per-line counts are now weighted; the output incorporates both
frequency and a measure of magnitude.

# Typical uses

This technique is often useful when you already know something -- e.g. a
general-purpose profiler showed that a particular function is hot -- but you
want to know more.

- Exactly how many times are paths X, Y and Z executed? For example, how often
  do lookups succeed or fail in data structure D? Print an identifying string
  each time a path is hit.
- How many times does loop L iterate? What does the loop count distribution
  look like? Is it executed frequently with a low loop count, or infrequently
  with a high loop count, or a mix? Print the iteration count before or after
  the loop.
- How many elements are typically in hash table H at this code location? Few?
  Many? A mixture? Print the element count.
- What are the contents of vector V at this code location? Print the contents.
- How many bytes of memory are used by data structure D at this code location?
  Print the size.
- Which call sites of function F are the hot ones? Print an identifying string
  at the call site.

Then use `counts` to aggregate the data. Often this domain-specific data is
critical to fully optimize hot code.

# Worse is better

Print statements are an admittedly crude way to get this kind of information,
profligate with I/O and disk space. In many cases you could do it in a way that
uses machine resources much more efficiently, e.g. by creating a small table
data structure in the code to track frequencies, and then printing that table
at program termination.

But that would require:
- writing the custom table (collection and printing);
- deciding where to define the table;
- possibly exposing the table to multiple modules;
- deciding where to initialize the table; and
- deciding where to print the contents of the table.

In a large program that can be a pain.

Alternatively, sometimes you want information that a general-purpose profiler
could give you, but running that profiler on your program is a hassle because
the program you want to profile is actually layered under something else, and
setting things up properly takes effort.

In contrast, inserting print statements is trivial. Any measurement can be set
up in no time at all. (Recompiling is often the slowest part of the process.)
This encourages experimentation. You can also kill a running program at any
point with no loss of profiling data.

Don't feel guilty about wasting machine resources; this is temporary code. I
sometimes end up with output files that are gigabytes in size. But `counts` is
fast because it's so simple. Let the machine do the work for you. (It does help
if you have a machine with an SSD.)

# Tips

Use unbuffered output for the print statements. In C and C++ code, use
`fprintf(stderr, ...)`. In Rust code use `eprintln!`.

Pipe the stderr output to file, e.g. `firefox 2> log`.

Sometimes programs print other lines of output to stderr that should be ignored
by `counts`. (Especially if they include integer IDs that `counts -w` would
interpret as weights!) So I usually prepend all my logging lines with a short
identifier, and then use `grep $ID log | counts` to ignore the other lines.
Sometimes I'll use more than one prefix, and then I can grep for each prefix
individually or all together.

Occasionally output lines get munged together when multiple print statements
are present. Because there are typically many lines of output, having a few
garbage ones almost never matters.

It's often useful to use both `counts` and `counts -w` on the same log file;
each one gives different insights into the data.

To find which call sites of a function call are hot, you can instrument the
call sites directly. But it's easy to miss one, and the same print statements
need to be repeated multiple times. An alternative is to add an extra string or
integer argument to the function, pass in a unique value from each call site,
and then print that value within the function.

It's occasionally useful to look at the raw logs as well as the output of
`counts`, because the sequence of output lines can be informative.

