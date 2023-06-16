## FLAC 文件结构（仅详细介绍元数据部分）

```text
Header(4 Bytes)
    固定值为 `fLaC` Hex: `66 4C 62 43`
Metadata-Block(注意：强制第一个块为 STREAM_INFO)
    标头(4 Bytes)
        1 Byte
            Bit 1: 标识是否是最后一个块, 最后一块为 1，否则为 0
            Bit 7: 标识该块是什么数据的代码
        3 Bytes
            表示该块的长度，使用 BigEndian 存储
    数据
        参阅[数据块定义](#数据块定义)
Metadata-Block *
    同上
Frame +
    此处不作阐述
```

## Header ID

- 0: STEAM INFO
- 1: PADDING
- 2: APPLICATION
- 3: SEEK TABLE
- 4: VORBIS COMMENT
- 5: CUESHEET
- 6: PICTURE
- 7-126: reserved
- 127: invalid

## 数据块定义

### Vorbis 数据块结构

注意下方内容使用 `Little Endian`

```text
Vendor
    4 Bytes 表示该字符串长度
    nn Bytes
Length
    4 Bytes 表示列表项的数量
Comment +
    4 Bytes 表示该项字符串的长度
    nn Bytes 键值对，使用 "=" 分割
```

### Picture 数据块结构

```text
Type
    4 Bytes 表示图片类型
Mime
    4 Bytes 表示 Mime 长度
    nn Bytes 图片 Mime 字符串 (ASCII)
Description
    4 Bytes 表示 Description 长度
    nn Bytes 图片 Description 字符串 (UTF-8)
Width
    4 Bytes 图片宽度
Height
    4 Bytes 图片高度
Color-Depth
    4 Bytes 色彩深度
Indexed-Color
    4 Bytes 索引颜色
Data
    4 Bytes 图片内容的长度
    nn Bytes 图片内容
```

## 参考

[https://xiph.org/flac/format.html](https://xiph.org/flac/format.html)