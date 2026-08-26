# utf8sten

short `u8s`

**disclaimer**:
  - output produced by encoding any data is ___not___ intended to have any meaning in any language,

    output ___only___ stores in itself original encoded data,

    if result of encoding has some meaning in some language, it's just coincidence.

encoding of raw data in unicode symbols using range U+8000 (which is valid from U+8000 to U+8FFF included),
which allows to store 1.5 bytes of raw data in 1 symbol, resulting in ratio of `bytes:symbols` be `3:2`,
resulting in 33.(3)% length reduction (note that output symbols are not ascii ∴ will take > 1 byte)


it wasn't designed to reduce raw byte size of the data encoded, rather to make X bytes of raw data become < X symbols (`symbols`<`bytes`).


on social platforms or in chats you usually can't send raw binary data and the best thing you can use as workaround is base64

but it will produce `symbols` > `bytes`, which will reach limit of message length much faster.
`u8s` on the other hand will produce `symbols` < `bytes`, which will allow you to send larger chunks of raw binary data
on platforms where message length is counted in symbols and not in bytes.

also that can be useful in combination with some compression (eg. gzip), since you can compress any data(including long text messages) and then encode it using `u8s` to be able to send it with additional length reduction of 33.(3)%.


no, it's not better than base64, it's just for different purpose.


## version 2
does the same but uses different range of unicodes U+20000 (from U+20000 to U+2A6DF included, which is also valid), with `symbols:bytes` ratio being `2:1`,
but with some limitation such as working reliably only with ascii symbols <= 0xA6, and with other data it's just gamble.

also on some platforms symbols from range U+20000 are counted as 2 symbols, resulting in ratio `symbols:bytes` = `1:1`, which is worse than regular `u8s`.
