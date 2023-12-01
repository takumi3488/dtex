# DTeX

良い感じのシンタックスでLaTeX変換が出来るやつです。

## インストール

cargoが必要です。

```
$ cargo install dtex
```

## 使い方

`main.d.tex`

```tex
config:
  fontsize: 11pt
  packages:
    - amsmath
    - amssymb
    - amsfonts
    - ascmac
    - bm
    - dvipdfmx.graphicx
    - here
    - physics
    - siunitx
    - comment
cover:
  title: "Example"
  author: "Author"
  date: "2020-01-01"
---
@@align
e^{i\pi} = @cos \pi@ + i@sin \pi@
=-1

@@csv ccc
example table
test1,test2,test3
$@SI 163 cm@$,$@SI 171 cm@$,$@SI 178 cm@$
```

```
$ dtex main.d.tex
```

ファイルの拡張子は `.d.tex` にしてください。
