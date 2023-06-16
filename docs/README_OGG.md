## OGG 文件结构

OGG 由多个页面组成

## OGG Page 结构

```text
Header:
    Signature
        4 Bytes: 固定值为 `OggS`, Hex: `4F  67 67 53`
    Version
        1 Byte: 版本，通常值为 0x0
    Flags
        1 Byte: 标识
            `0x00`(Unset) 表示这个页面包含新的数据包
            `0x01`(Continuation) 表示这个页面是上一个页面的继续的数据包
            `0x02`(BOS, Begin of Stream) 表示页面是第一个页面
            `0x04`(EOS, End of Stream) 表示页面是最后一个页面
    Granule-Position
        8 Bytes: 此处不作阐述
    Serial-Number
        4 Bytes: 此处不作阐述
    Sequence-Number
        4 Bytes: 此处不作阐述
    Checksum
        4 Bytes: 该页面的校验值，由 Crc32 计算而来（ checksum 设置为 0，排除 Segment-Table ）
    Total-Segments:
        1 Byte: 下方 Segment-Table 的长度
    Segment-Table:
        nn Bytes: 表示 Packet 的长度，上方的 Total-Segments 长度决定字节数，所有字节进行“累加” 
Packet
    参阅[OGG Packet 结构](#OGG Packet 结构)
```

## OGG Packet 结构

注意，Packet 可能由多个 Page 构成，参阅上方的 `Flags` 字段

Packet 格式由具体地编音频码类型所决定，这里给出 Vorbis 和 Opus 两种类型的包

### OGG Vorbis Packet

一个 Vorbis 流以三个强制的 Header Packet 开始，这三个 Header Packet 按顺序为：

1. Identification-Header
2. Comments-Header
3. Setup-Header

每个 Header Packet 都以相同的 Header 字段开始

```text
Packet_Type
    1 Byte: Packet 类型
        `0x01` Identification Header
        `0x03` Comments Header
        `0x05` Setup Header
        `0x00` 已经其他为偶数的类型都是音频 Packet
    6 Bytes: 字符 `vorbis`
```

```text
Identification
    Common
        1 Byte: 0x1 标识这是 Identification 头
        7 Bytes: Vorbis
    Few Audio Information
        1 Byte: Version，始终为 0x00
        1 Byte: 音频通道
        4 Bytes: 采样率
        4 Bytes: 最大采样率
        4 Bytes: 宣称的采样率
        4 Bytes: 最小采样率
        1 Byte: blocksize
        1 Byte: framing_flag
Comments
    Common
        1 Byte: 0x3 标识这是 Identification 头
        7 Bytes: Vorbis    
    Vendor
        4 Bytes 表示该字符串长度
        nn Bytes
    Length
        4 Bytes 表示列表项的数量
    Comment +
        4 Bytes 表示该项字符串的长度
        nn Bytes 键值对，使用 "=" 分割
```

### OGG Opus Packet

一个 Opus 流以两个强制的头开始，一个 Identification 头一个 Comment 头

```text
Identification
    Common
        8 Bytes: 固定值: "OpusHead"，Hex: `4F 70 75 73 54 61 67 73`
    Few Audio Information
        1 Byte: Version，始终为 0x01
        1 Byte: 输出通道数
        2 Bytes: Pre-Skip
        4 Bytes: Input Sample Rate (HZ)
        2 Bytes: Output Gain, little endian
        1 Byte: Channel Mapping Family 
        nn Bytes: Channel Mapping Table, must be omitted when the channel mapping family is 0
Comments
    Common
        8 Bytes: 固定值: "OpusHead"，Hex: `4F 70 75 73 54 61 67 73`
    Vendor
        4 Bytes 表示该字符串长度
        nn Bytes
    Length
        4 Bytes 表示列表项的数量
    Comment +
        4 Bytes 表示该项字符串的长度
        nn Bytes 键值对，使用 "=" 分割
```

## 参考

[Ogg](https://www.rfc-editor.org/rfc/rfc3533.html)

[OggVorbis](https://xiph.org/vorbis/doc/Vorbis_I_spec.html)

[OggOpus](https://datatracker.ietf.org/doc/html/rfc7845)