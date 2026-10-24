# Project images

`telegram.jpg` is an unmodified screenshot of the real bot. `transmitter.jpg` is an AI-retouched photograph of the real
transmitter: tabletop dust was removed, the background softened, and exposure adjusted. The Desktop originals were left
unchanged. `social-preview.jpg` is a 1280 × 640 banner composed from the retouched photo.

Both edits used the built-in image generation tool, followed by JPEG export for repository size. These images document
the prototype’s appearance; they are not wiring references.

To activate the banner, open the repository’s **Settings → General → Social preview → Edit → Upload an image** and select
`social-preview.jpg`. The GitHub CLI does not expose this upload.

## Transmitter retouch prompt

Use case: precise-object-edit. Asset type: documentary hardware photo for a GitHub project README. Edit target: the
provided real photo of a small homemade USB-powered infrared transmitter on a dark wood tabletop. Remove visible dust,
lint and small surface blemishes from the tabletop; soften distracting tabletop texture in foreground and background
very subtly with natural shallow depth of field. Improve exposure and white balance modestly for a clean natural photo.
Invariants: preserve the exact real device, its red translucent enclosure, metal clip, USB connector, cables, taped
joints, resistors, sensor/LED, circuit geometry, placement, perspective, framing and shadows. Do not redesign, add,
remove or improve hardware components. Keep an honest homemade prototype appearance. No added text, no watermark. The
requested change is tabletop cleanup, with gentle background softening only.

## Social preview prompt

Use case: compositing. Asset type: GitHub social preview banner, 1280 by 640 landscape (2:1). Create a restrained clean
editorial banner for the real personal project IR Remote, using this retouched device photo as the photographic content.
Layout: dark warm charcoal background with left third containing clear large off-white title 'IR Remote' and smaller
subtitle 'An air conditioner on Telegram.' and a small final line 'Rust · ESP8266 · Encrypted UDP'. Use a modern simple
sans-serif, generous spacing. The right two thirds show the actual provided homemade transmitter photograph, with a soft
gradient transition into the left text panel. Preserve the red enclosure, USB connector, wires, soldered board and
components in the supplied photograph; don't redesign hardware or invent new hardware. No logos, no badges, no icons, no
extra objects, no watermarks. Keep text legible and within generous safe margins; polished but understated.
