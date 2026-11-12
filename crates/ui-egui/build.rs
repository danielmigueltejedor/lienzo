//! Packs the translation catalogs (`src/i18n/*.tsv`) into `OUT_DIR/i18n/` for
//! `i18n::LANGUAGES` (see [`pack_catalogs`]): stored that way they take ~0.5 MB of the binary
//! instead of ~2.2 MB, which keeps the web build under its 24 MiB size gate
//! (packaging/web/package.sh). A catalog is unpacked the first time its language is used.

use std::fmt::Write as _;

fn main() {
    pack_catalogs();
    deflate_egui_fonts();
}

/// Deflate egui's default fonts (the `epaint_default_fonts` crate) into `OUT_DIR/egui_fonts/` for
/// `egui_fonts.rs`. The app builds egui without `default_fonts`, which would embed the same 1.4 MB
/// uncompressed; deflated they take 0.78 MB (the web build's 24 MiB size gate).
fn deflate_egui_fonts() {
    use std::io::Write as _;
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("egui_fonts");
    // `egui-icons` is unconditional in 0.37 (a handful of private-use symbols, 3.5 kB); the two
    // emoji fonts come through `monochrome_emoji_fonts` in Cargo.toml.
    let fonts = [
        ("Hack-Regular", epaint_default_fonts::HACK_REGULAR),
        ("egui-icons", epaint_default_fonts::EGUI_ICONS),
        ("NotoEmoji-Regular", epaint_default_fonts::NOTO_EMOJI_REGULAR),
        ("Ubuntu-Light", epaint_default_fonts::UBUNTU_LIGHT),
        ("emoji-icon-font", epaint_default_fonts::EMOJI_ICON),
    ];
    let result = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(&out)?;
        for (name, bytes) in fonts {
            let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
            z.write_all(bytes)?;
            std::fs::write(out.join(format!("{name}.ttf.deflate")), z.finish()?)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        println!("cargo::error=deflating egui's default fonts into {}: {e}", out.display());
    }
}

/// Whether a catalog line is an entry (`context<TAB>source<TAB>translation`, source and
/// translation non-empty, as `i18n::catalog::parse_entries` reads it), and its key and translation.
fn catalog_entry(line: &str) -> Option<(&str, &str)> {
    if line.starts_with('#') {
        return None;
    }
    let mut cols = line.splitn(3, '\t');
    let (ctx, src, tr) = (cols.next()?, cols.next()?, cols.next()?);
    let key = line.get(..ctx.len() + 1 + src.len())?;
    (!src.is_empty() && !tr.is_empty() && !tr.contains('\t')).then_some((key, tr))
}

/// Pack every `src/i18n/*.tsv` into `OUT_DIR/i18n/`, losslessly (`i18n::unpack` restores each file
/// byte for byte). Every catalog repeats the same English sources, so they are stored once:
///
/// - `keys.deflate`: every entry's `context<TAB>source`, one per line, in order of first
///   appearance (the largest catalog first).
/// - `<code>.tsv.deflate`: the catalog, each entry line replaced by `\x01<translation>` when its key
///   follows the previous entry's in `keys`, else `\x02<offset><TAB><translation>` (the offset from
///   there). Other lines are kept; one that starts with \x01, \x02 or \x03 gets a \x03 prefix.
///
/// Both are deflated. A catalog that can't be written is a build error (`include_bytes!` would
/// fail on it anyway, with a less helpful message).
fn pack_catalogs() {
    use std::io::Write as _;
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/i18n");
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("i18n");
    println!("cargo::rerun-if-changed={}", src.display());
    let deflate = |bytes: &[u8]| -> std::io::Result<Vec<u8>> {
        let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
        z.write_all(bytes)?;
        z.finish()
    };
    let result = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(&out)?;
        let mut catalogs = Vec::new();
        for entry in std::fs::read_dir(&src)? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()).filter(|n| n.ends_with(".tsv")) else { continue };
            println!("cargo::rerun-if-changed={}", path.display());
            catalogs.push((name.to_owned(), std::fs::read_to_string(&path)?));
        }
        // Largest first, then by name: the key order (and so the output) is reproducible.
        catalogs.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
        let mut keys: Vec<&str> = Vec::new();
        let mut index = std::collections::HashMap::new();
        for line in catalogs.iter().flat_map(|(_, text)| text.split('\n')) {
            if let Some((key, _)) = catalog_entry(line) {
                index.entry(key).or_insert_with(|| {
                    keys.push(key);
                    keys.len() - 1
                });
            }
        }
        std::fs::write(out.join("keys.deflate"), deflate(keys.join("\n").as_bytes())?)?;
        for (name, text) in &catalogs {
            let mut packed = String::with_capacity(text.len());
            let mut next = 0usize;
            for (n, line) in text.split('\n').enumerate() {
                if n > 0 {
                    packed.push('\n');
                }
                match catalog_entry(line) {
                    Some((key, tr)) => {
                        let i = index.get(key).copied().unwrap_or_default();
                        if i == next {
                            packed.push('\x01');
                        } else {
                            let _ = write!(packed, "\x02{}\t", i as i64 - next as i64);
                        }
                        packed.push_str(tr);
                        next = i + 1;
                    }
                    None => {
                        if line.starts_with(['\x01', '\x02', '\x03']) {
                            packed.push('\x03');
                        }
                        packed.push_str(line);
                    }
                }
            }
            std::fs::write(out.join(format!("{name}.deflate")), deflate(packed.as_bytes())?)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        println!("cargo::error=packing the translation catalogs in {}: {e}", src.display());
    }
}
