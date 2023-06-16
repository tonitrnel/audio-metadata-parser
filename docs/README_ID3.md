## ID3 结构

```text
Header
    3 Bytes: 固定值为 `ID3`，Hex: `49, 44, 03`
    2 Bytes: 版本，第一个字节为主要版本，第二个字节为修订号
    1 Byte: 标志位
        Bit 5: 测试标签
        Bit 6: 扩展标头
        Bit 7: 是不同步
    4 Bytes: 标签的总大小，不包含标头，Big endian
Extended Header(Optionally), 取决于 标志位
    4 Bytes: 扩展标头大小
    1 Byte: First Flag, 标注，如果 Bit 7 存在，则后面有 CRC32.
    1 Byte: Second Flag. Always 0
    4 Bytes: 填充大小
    4 Bytes(Optionally): CRC32，根据 First Flag 决定是否存在
Tags
    Frame+
        4 Bytes: ID，X、Y、Z 开头的 ID 是测试用的
        4 Bytes: Data size
        1 Byte: Flag
            Bit 7: Tag alter preservation
            Bit 6: File alter preservation
            Bit 5: Readonly
        1 Byte: Flag
            Bit 7: Compression
            Bit 6: Encryption
            Bit 5: Grouping identity
        1 Byte: Frame 内容编码方式
        nn Bytes: data
AudioData
Optionally, v1 Tags
```

参考：

[Id3v2.3.0](https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.3.0.html)