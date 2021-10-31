# stew3d
A disassembler for the [Stew 3000](https://github.com/stew2003/Stew-3000).


## Install
```bash
$ cargo build --release
$ sudo cp target/release/stew3d /usr/local/bin
```

Now, restarting your shell, you can invoke `stew3d` directly:
```bash
$ stew3d ./my-binary.3000.b

Disassembly of file `./my-binary.3000.b` (8 bytes)

00:    7f 0a    |   mvi 10, a
02:    bc 05    |   call l0
04:    c7       |   hlt
05:             | l0:
05:    0c 04    |   addi 4, a
07:    bd       |   ret
```
