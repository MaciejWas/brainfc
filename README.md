# brainfc
A compiler for the brainfuck programming language

## How to run
The commands below compile the mandelbrot set generator made by Erik Bosman.

```
$ git clone https://github.com/MaciejWas/brainfc
$ cd brainfc
$ cargo build
$ curl https://raw.githubusercontent.com/erikdubbelboer/brainfuck-jit/master/mandelbrot.bf > mandelbrot.bf
$ cargo run -- ./mandelbrot.bf
```
