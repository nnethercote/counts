# Overview

`counts` is a command line tool for ad hoc profiling. It tallies line
frequencies within text files, like an improved version of the Unix command
chain `sort | uniq -c`.

You can use it in combination with logging print statements in a program of
interest to obtain invaluable, domain-specific profiling data.

# Installing

To install from [crates.io](https://crates.io/):

> `cargo install counts`

This requires Rust 1.56 or later. The compiled binary will be put into
`~/.cargo/bin/`.

To update the installation:

> `cargo install --force counts`

# A simple usage example

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

Alternatively, when invoked with the `-i` flag, it assigns each line an
integral weight, determined by the last integer that appears on the line (or 1
if there is no such integer).  On the same input, `counts -i` produces the
following output.
```
30 counts (weighted integral)
(  1)       16 (53.3%, 53.3%): d 4
(  2)        9 (30.0%, 83.3%): c 3
(  3)        4 (13.3%, 96.7%): b 2
(  4)        1 ( 3.3%,100.0%): a 1
```
The total and per-line counts are now weighted; the output incorporates both
frequency and a measure of magnitude.

The `-f` flag can be used for fractional weights, which can be integers or
fractional numbers of the form `mm.nn`.

Negative weights are allowed. In the output, each entry is sorted by the
absolute value of its aggregate weight. This means that both large positive and
large negative entries will show up near the top.

Sometimes you want to group together lines that have different weights but are
otherwise the same. The `-e` flag can be used to erase weights after applying
them, by replacing them with `NNN`. Consider the following input.
```
a 1
b 2
a 3
b 4
a 5
```
`counts -i` will produce the following output, which is uninteresting.
```
15 counts (weighted integral)
(  1)        5 (33.3%, 33.3%): a 5
(  2)        4 (26.7%, 60.0%): b 4
(  3)        3 (20.0%, 80.0%): a 3
(  4)        2 (13.3%, 93.3%): b 2
(  5)        1 ( 6.7%,100.0%): a 1
```
`counts -i -e` will produce the following output, where the different `a` and
`b` lines have been grouped together.
```
15 counts (weighted integral, erased)
(  1)        9 (60.0%, 60.0%): a NNN
(  2)        6 (40.0%,100.0%): b NNN
```

# A more complex usage example

As an example, I added print statements to Firefox's heap allocator so it
prints a line for every allocation that shows its category, requested size, and
actual size. A short run of Firefox with this instrumentation produced a 77 MB
file containing 5.27 million lines. `counts` produced the following output for
this file.
```
5270459 counts
( 1) 576937 (10.9%, 10.9%): small 32 (32)
( 2) 546618 (10.4%, 21.3%): small 24 (32)
( 3) 492358 ( 9.3%, 30.7%): small 64 (64)
( 4) 321517 ( 6.1%, 36.8%): small 16 (16)
( 5) 288327 ( 5.5%, 42.2%): small 128 (128)
( 6) 251023 ( 4.8%, 47.0%): small 512 (512)
( 7) 191818 ( 3.6%, 50.6%): small 48 (48)
( 8) 164846 ( 3.1%, 53.8%): small 256 (256)
( 9) 162634 ( 3.1%, 56.8%): small 8 (8)
( 10) 146220 ( 2.8%, 59.6%): small 40 (48)
( 11) 111528 ( 2.1%, 61.7%): small 72 (80)
( 12) 94332 ( 1.8%, 63.5%): small 4 (8)
( 13) 91727 ( 1.7%, 65.3%): small 56 (64)
( 14) 78092 ( 1.5%, 66.7%): small 168 (176)
( 15) 64829 ( 1.2%, 68.0%): small 96 (96)
( 16) 60394 ( 1.1%, 69.1%): small 88 (96)
( 17) 58414 ( 1.1%, 70.2%): small 80 (80)
( 18) 53193 ( 1.0%, 71.2%): large 4096 (4096)
( 19) 51623 ( 1.0%, 72.2%): small 1024 (1024)
( 20) 45979 ( 0.9%, 73.1%): small 2048 (2048)
```
Unsurprisingly, small allocations dominate. But what happens if we weight each
entry by its size? `counts -i` produced the following output.
```
2554515775 counts (weighted integral)
( 1) 501481472 (19.6%, 19.6%): large 32768 (32768)
( 2) 217878528 ( 8.5%, 28.2%): large 4096 (4096)
( 3) 156762112 ( 6.1%, 34.3%): large 65536 (65536)
( 4) 133554176 ( 5.2%, 39.5%): large 8192 (8192)
( 5) 128523776 ( 5.0%, 44.6%): small 512 (512)
( 6) 96550912 ( 3.8%, 48.3%): large 3072 (4096)
( 7) 94164992 ( 3.7%, 52.0%): small 2048 (2048)
( 8) 52861952 ( 2.1%, 54.1%): small 1024 (1024)
( 9) 44564480 ( 1.7%, 55.8%): large 262144 (262144)
( 10) 42200576 ( 1.7%, 57.5%): small 256 (256)
( 11) 41926656 ( 1.6%, 59.1%): large 16384 (16384)
( 12) 39976960 ( 1.6%, 60.7%): large 131072 (131072)
( 13) 38928384 ( 1.5%, 62.2%): huge 4864000 (4866048)
( 14) 37748736 ( 1.5%, 63.7%): huge 2097152 (2097152)
( 15) 36905856 ( 1.4%, 65.1%): small 128 (128)
( 16) 31510912 ( 1.2%, 66.4%): small 64 (64)
( 17) 24805376 ( 1.0%, 67.3%): huge 3097600 (3100672)
( 18) 23068672 ( 0.9%, 68.2%): huge 1048576 (1048576)
( 19) 22020096 ( 0.9%, 69.1%): large 524288 (524288)
( 20) 18980864 ( 0.7%, 69.9%): large 5432 (8192)
```
This shows that the cumulative count of allocated bytes (2.55GB) is dominated
by a mixture of larger allocation sizes.

This example gives just a taste of what `counts` can do.

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
  Print the byte size.
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

That is a pain, especially in a large program you don't fully understand.

Alternatively, sometimes you want information that a general-purpose profiler
could give you, but running that profiler on your program is a hassle because
the program you want to profile is actually layered under something else, and
setting things up properly takes effort.

In contrast, inserting print statements is trivial. Any measurement can be set
up in no time at all. (Recompiling is often the slowest part of the process.)
This encourages experimentation. You can also kill a running program at any
point with no loss of profiling data.

Don't feel guilty about wasting machine resources; this is temporary code. You
might sometimes end up with output files that are gigabytes in size. But
`counts` is fast because it's so simple. Let the machine do the work for you.
(It does help if you have a machine with an SSD.)

# Ad Hoc Profiling

For a long time I have, in my own mind, used the term ad hoc profiling to
describe this combination of logging print statements and frequency-based
post-processing. Wikipedia [defines](https://en.wikipedia.org/wiki/Ad_hoc) "ad
hoc" as follows.

> In English, it generally signifies a solution designed for a specific problem
> or task, non-generalizable, and not intended to be able to be adapted to
> other purposes

The process of writing custom code to collect this kind of profiling data — in
the manner I disparaged in the previous section — truly matches this definition
of "ad hoc".

But `counts` is valuable specifically because it makes this type of custom
profiling less ad hoc and more repeatable. I should arguably call it
"generalized ad hoc profiling" or "not so ad hoc profiling", but those names
don’t have quite the same ring to them.

# Tips

Use unbuffered output for the print statements. In C and C++ code, use
`fprintf(stderr, ...)`. In Rust code use `eprintln!` or `dbg!`.

Pipe the stderr output to file, e.g. `firefox 2> log`.

If you are generating large amounts of data, piping your data through a fast 
compressor could be convenient. `zstd` can process gigabytes per second, 
while saving more than 90% on disk usage. Use `firefox 2>&1 >/dev/null | zstd 
--fast -o log.zst` to write and `zstd -d -c log.zst | counts` to process.

Sometimes programs print other lines of output to stderr that should be ignored
by `counts`. (Especially if they include integer IDs that `counts -i` would
interpret as weights!) Prepend all logging lines with a short identifier, and
then use `grep $ID log | counts` to ignore the other lines. If you use more
than one prefix, you can grep for each prefix individually or all together.

Occasionally output lines get munged together when multiple print statements
are present. Because there are typically many lines of output, having a few
garbage ones almost never matters.

It's often useful to use both `counts` and `counts -i` on the same log file;
each one gives different insights into the data.

To find which call sites of a function call are hot, you can instrument the
call sites directly. But it's easy to miss one, and the same print statements
need to be repeated multiple times. An alternative is to add an extra string or
integer argument to the function, pass in a unique value from each call site,
and then print that value within the function.

It's occasionally useful to look at the raw logs as well as the output of
`counts`, because the sequence of output lines can be informative.

