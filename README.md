# brainfc
A compiler for the brainfuck programming language. Uses LLVM as it's backend and makes couple easy optimizations:

 * Sets the main pointer to zero when encountering `[-]`
 * Detects "multiplication loops", for example `[>++>>+++<<<-]` which is roughly equivalent to pseudocode
 ``` 
 arr[ptr+1] = arr[ptr] * 2; 
 arr[ptr+2] = arr[ptr] * 3; 
 arr[ptr] = 0
 ```


## How to run
The commands below compile the mandelbrot set generator made by Erik Bosman.

```
$ git clone https://github.com/MaciejWas/brainfc
$ cd brainfc
$ sh ./benchmark.sh
```

## Usage
```
Usage: brainfc [OPTIONS] <path>

Arguments:
  <path>

Options:
  -o, --output <OUTPUT>
      --show-parsed
      --show-optimized
      --show-llvm-ir
  -h, --help             Print help
  -V, --version          Print version
```
