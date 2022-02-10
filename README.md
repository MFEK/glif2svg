# Convert UFO glyph files (`.glif`) to SVG

There exists already an `svg2glif`, but for some reason not the opposite operation. My MFEKglif editor treats `.glif` files as first-class entities, so it seemed obvious to me I'd want to edit them with Inkscape, etc sometimes.

I originally wrote this in Python ([ctrlcctrlv/glif2svg](https://github.com/ctrlcctrlv/glif2svg)) but it was too slow for the huge amount I was using it in FRBAmericanCursive's build. So, I ported it to Rust. Also, I decided to make it part of the MFEK suite of programs, since it heavily relies on MFEK libaries like glifparser and mfek-ipc.

```
glif2svg 1.0.0
Fredrick R. Brennan <copypasteⒶkittens⊙ph>; MFEK Authors
Convert between glif to SVG

USAGE:
    glif2svg [FLAGS] [OPTIONS] <input>

FLAGS:
    -B, --no-viewbox    Don't put viewBox in SVG
    -M, --no-metrics    Don't consider glif's height/width when writing SVG, use minx/maxx/miny/maxy
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -o, --output <output_file>     The path to the output file. If not provided, or `-`, stdout.
                                   
                                   
    -F, --fontinfo <fontinfo>      fontinfo file (for metrics, should point to fontinfo.plist path if you are using an
                                   unparented glif, a glif not in a parent UFO font)
    -p, --precision <precision>    Float precision [default: 6]

ARGS:
    <input>    The path to the input file.
```

## Requirements

This is a normal Rust build; however the final binary will expect you to have [MFEKmetadata](https://github.com/MFEK/metadata) in your path as well.

## License

```
Copyright 2021 Fredrick R. Brennan & MFEK Authors

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this software or any of the provided source code files except in compliance
with the License.  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied.  See the License for the
specific language governing permissions and limitations under the License.
```

**By contributing you release your contribution under the terms of the license.**
