# App fonts

The app expects these files here:

    assets/fonts/OpenSans_SemiCondensed-SemiBold.ttf
    assets/fonts/Inter-SemiBold.ttf.b64
    assets/fonts/Roboto-Medium.ttf.b64

Inter and Roboto are stored base64-encoded as text (the `.ttf.b64` files decode to the exact TTF
bytes; `src/fonts.rs` decodes them at startup) because the tooling that carries this repo handles
text only.

The settings' `UI font:` and `Browser font:` picks choose between them independently. Open Sans
(SemiCondensed width, SemiBold weight) and Inter (SemiBold) are licensed under the SIL Open Font
License (OFL; `OFL.txt` and `Inter-LICENSE.txt`); Roboto (Medium) is licensed under the Apache
License 2.0 (`Roboto-LICENSE.txt`). Inter is from the rsms/inter release v4.1, Roboto from the
google/roboto release v2.138.

Get it from Google Fonts: family "Open Sans", style SemiCondensed SemiBold
(<https://fonts.google.com/specimen/Open+Sans>). Download the family and take the static
`OpenSans_SemiCondensed-SemiBold.ttf` from the archive.

The font is committed here together with its license (`OFL.txt`, from the official Open Sans repository).
Source: the static file from the Google Fonts "Open Sans" download bundle (version 196805,
sha256 `d4baec1280b3c9be62f48abfd8acfd09a04958d663d316d16c4e567a32440828`).

`build.rs` detects the file: with the file present it is embedded and becomes the first
`Proportional` font; without it the app builds and runs with egui's built-in fonts. After adding or removing the
file, `cargo build` picks the change up automatically.
