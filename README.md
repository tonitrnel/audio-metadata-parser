# Audio Metadata Parser

A metadata parser for music files in Node.js, implemented in Rust, supporting mp3, flac, and ogg formats.

If you wish to use this in a browser, simply copy the parser code from my other project [metadata-parser](https://github.com/tonitrnel/synclink/tree/dev/web/src/components/audio-player/metadata-parser), which follows the same logic.


## Installation

```bash
npm install @ptdgrp/audmetap-binding
```

## Usage

The following TypeScript example demonstrates how to use the parser to extract metadata from a music file:

```typescript
import { parse } from "@ptdgrp/audmetap-binding";

// Read the file as a buffer
const buf = await fs
    .readFile("<PATH>")
    .then((r) => Uint8Array.from(r));

// Parse the metadata from the buffer
const metadata = parse(buf);

// Output the extracted metadata
console.log(metadata.title);
console.log(metadata.album);
console.log(metadata.artist);

// If cover art is available, output the cover information and save the image
if (metadata.cover) {
    console.log(metadata.cover.description);
    console.log(metadata.cover.mime);

    // Save the cover image file with an appropriate extension
    await fs.writeFile(`cover.${metadata.cover.mime.slice('/')[1] || 'png'}`, new Uint8Array(metadata.cover.data))
}
```

## LICENSE

For licensing information, see the [LICENSE](LICENSE) file.