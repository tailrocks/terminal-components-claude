# Frozen tag 'visual-baseline' — rendered capture foreground histograms

Measured with `grep -o '38;5;[0-9]*' | sort | uniq -c | sort -rn` (256, 16) and
`grep -o '38;2;[0-9]*;[0-9]*;[0-9]*' | sort | uniq -c | sort -rn` (truecolor) over the tag's own
rendered captures.

## `visual-baseline:snapshots/showcase/pages/overview/100x30/256.ansi`

```
 203 38;5;231
  27 38;5;245
  22 38;5;242
  19 38;5;239
  15 38;5;249
   9 38;5;232
   4 38;5;78
   3 38;5;238
```

## `visual-baseline:snapshots/showcase/pages/overview/100x30/truecolor.ansi`

```
 204 38;2;255;255;255
  25 38;2;138;138;138
  22 38;2;115;115;115
  19 38;2;71;71;71
  15 38;2;179;179;179
   9 38;2;17;17;17
   4 38;2;72;224;84
   3 38;2;58;58;58
   2 38;2;128;128;128
```

## `visual-baseline:snapshots/showcase/pages/overview/100x30/16.ansi`

```
 130 38;5;15
  64 38;5;7
  22 38;5;8
  10 38;5;0
   4 38;5;10
```

