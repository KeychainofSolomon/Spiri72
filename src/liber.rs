//! The book: a PDF handed by the Operator, read page by page.
//!
//! A page that carries its own text is read as it stands (no model is asked). A page
//! that is only a picture, such as a scan, is drawn and shown to the spirit's eyes, which
//! copy out its writing (OCR). Nothing here runs any code from the PDF: it is only drawn
//! and its letters gathered.

use hayro::hayro_interpret::font::{Glyph, GlyphRun};
use hayro::hayro_interpret::hayro_cmap::BfString;
use hayro::hayro_interpret::{
    interpret_page, BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image,
    ImageDrawProps, InterpreterCache, InterpreterSettings, SoftMask,
};
use hayro::hayro_syntax::Pdf;
use hayro::kurbo::{BezPath, Point, Rect};
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{render, PixmapSettings, RenderCache, RenderSettings};

/// The most pages the eyes will read in one book (each takes the eyes a while).
pub const MAX_SEEN_PAGES: usize = 20;
/// The longest side of a page drawn for the eyes, in pixels.
const SIGHT_PIXELS: f32 = 1280.0;
/// A page with fewer letters than this is taken to be a picture, not text.
const MIN_LETTERS: usize = 16;

/// One letter as drawn: where it stands on the page, how tall its font is.
struct Letter {
    x: f64,
    y: f64,
    size: f64,
    /// Where the letter ends: the next letter of its word starts about here.
    end: f64,
    text: String,
}

/// A device that draws nothing and only gathers the letters shown.
#[derive(Default)]
struct Letters(Vec<Letter>);

impl<'a> Device<'a> for Letters {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'a>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn draw_glyph_run(&mut self, run: &GlyphRun<'_, 'a>, props: DrawProps<'a>, _: &DrawMode) {
        for g in run.glyphs() {
            let text = match Glyph::as_unicode(g) {
                Some(BfString::Char(c)) => c.to_string(),
                Some(BfString::String(s)) => s,
                None => continue,
            };
            // A glyph's own transform places it in text space only; the page's current
            // transform (here, `props.transform`) places it on the page. A page that flips
            // its space (`1 0 0 -1 … cm`, as in Behave) reads upside down without it.
            let t = props.transform * g.transform();
            let o = t * Point::ZERO;
            let [_, _, c, d, _, _] = t.as_coeffs();
            let size = (c.hypot(d) * 1000.0).max(0.1);
            let advance = match &**g {
                Glyph::Outline(o) => o.advance_width(),
                Glyph::Type3(t) => t.advance_width(),
            };
            let end = match advance {
                Some(a) if a > 0.0 => (t * Point::new(a as f64, 0.0)).x,
                _ => o.x + size * 0.5,
            };
            self.0.push(Letter {
                x: o.x,
                y: o.y,
                size,
                end,
                text,
            });
        }
    }
    fn draw_image(&mut self, _: Image<'a, '_>, _: ImageDrawProps<'a>) {}
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
}

/// One line's letters parted where a wide gap falls (more than one and a half letters' height).
fn segments(line: &[Letter]) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..line.len() {
        let (p, l) = (&line[i - 1], &line[i]);
        if l.x - p.end > p.size.min(l.size) * 1.5 {
            out.push(start..i);
            start = i;
        }
    }
    out.push(start..line.len());
    out
}

/// Where a page of two columns parts them: the middle of the gutter, if at least a
/// third of its lines (and no fewer than four) have a wide gap near the middle of the
/// page. `None` for a page of one column.
fn gutter(lines: &[Vec<Letter>]) -> Option<f64> {
    let left = lines
        .iter()
        .flatten()
        .map(|l| l.x)
        .fold(f64::INFINITY, f64::min);
    let right = lines
        .iter()
        .flatten()
        .map(|l| l.end)
        .fold(f64::NEG_INFINITY, f64::max);
    let width = right - left;
    if width.is_nan() || width <= 0.0 {
        return None;
    }
    let mut gaps: Vec<f64> = Vec::new();
    for line in lines {
        for w in segments(line).windows(2) {
            let a = line[w[0].end - 1].end;
            let b = line[w[1].start].x;
            let mid = (a + b) / 2.0;
            if (left + width * 0.3..left + width * 0.7).contains(&mid) {
                gaps.push(mid);
                break;
            }
        }
    }
    if gaps.len() < 4 || gaps.len() * 3 < lines.len() {
        return None;
    }
    gaps.sort_by(f64::total_cmp);
    Some(gaps[gaps.len() / 2])
}

/// One line's letters as text, a space where words part.
fn say_line(line: &[Letter], out: &mut String) {
    let mut prev: Option<&Letter> = None;
    for l in line {
        if let Some(p) = prev {
            let parted = l.x - p.end > p.size.min(l.size) * 0.15;
            if parted && !p.text.ends_with(' ') && !l.text.starts_with(' ') {
                out.push(' ');
            }
        }
        out.push_str(&l.text);
        prev = Some(l);
    }
    let kept = out.trim_end().len();
    out.truncate(kept);
    out.push('\n');
}

/// The letters set in lines: top to bottom, left to right, a space where words part.
/// A space is read wherever a letter starts well past where the one before it ended.
/// A page of two columns is read down the left column, then down the right: a line
/// that crosses the gutter (a heading) is read where it falls in the left column.
fn set_in_lines(mut letters: Vec<Letter>) -> String {
    // Page space rises upwards: the top line has the greatest height.
    letters.sort_by(|a, b| b.y.total_cmp(&a.y).then(a.x.total_cmp(&b.x)));
    let mut lines: Vec<Vec<Letter>> = Vec::new();
    for l in letters {
        match lines.last_mut() {
            Some(line) if (line[0].y - l.y).abs() < line[0].size.min(l.size) * 0.5 => line.push(l),
            _ => lines.push(vec![l]),
        }
    }
    for line in &mut lines {
        line.sort_by(|a, b| a.x.total_cmp(&b.x));
    }
    let mut out = String::new();
    match gutter(&lines) {
        None => {
            for line in &lines {
                say_line(line, &mut out);
            }
        }
        Some(g) => {
            let mut right = String::new();
            for line in &lines {
                let cut = line.iter().position(|l| l.x >= g).unwrap_or(line.len());
                // a line whose letters cross the gutter with no gap is one line
                let crosses = cut > 0
                    && cut < line.len()
                    && line[cut].x - line[cut - 1].end
                        <= line[cut - 1].size.min(line[cut].size) * 1.5;
                if crosses {
                    say_line(line, &mut out);
                    continue;
                }
                if cut > 0 {
                    say_line(&line[..cut], &mut out);
                }
                if cut < line.len() {
                    say_line(&line[cut..], &mut right);
                }
            }
            out.push_str(&right);
        }
    }
    out.trim().to_string()
}

/// The pages of a book: for each, its own text if it has any.
pub struct Book {
    pdf: Pdf,
}

impl Book {
    pub fn open(bytes: &[u8]) -> Result<Book, String> {
        Pdf::new(bytes.to_vec())
            .map(|pdf| Book { pdf })
            .map_err(|e| match format!("{e:?}") {
                s if s.contains("ncrypt") => "this PDF is locked with a password".to_string(),
                _ => "this PDF cannot be opened (it may be damaged)".to_string(),
            })
    }

    pub fn page_count(&self) -> usize {
        self.pdf.pages().len()
    }

    /// The text the page carries, or none if it is only a picture.
    pub fn text_of(&self, idx: usize) -> Option<String> {
        let page = self.pdf.pages().get(idx)?;
        let cache = InterpreterCache::new();
        let (w, h) = page.render_dimensions();
        let mut ctx = Context::new(
            hayro::kurbo::Affine::IDENTITY,
            Rect::new(0.0, 0.0, w as f64, h as f64),
            &cache,
            page.xref(),
            InterpreterSettings::default(),
        );
        let mut letters = Letters::default();
        interpret_page(page, &mut ctx, &mut letters);
        let text = set_in_lines(letters.0);
        (text.chars().filter(|c| c.is_alphanumeric()).count() >= MIN_LETTERS).then_some(text)
    }

    /// The page drawn as a PNG for the eyes, its longest side `SIGHT_PIXELS`.
    pub fn picture_of(&self, idx: usize) -> Option<Vec<u8>> {
        let page = self.pdf.pages().get(idx)?;
        let (w, h) = page.render_dimensions();
        let scale = (SIGHT_PIXELS / w.max(h).max(1.0)).clamp(0.05, 8.0);
        let pixmap = render(
            page,
            &RenderCache::new(),
            &InterpreterSettings::default(),
            &RenderSettings::default(),
            &PixmapSettings {
                x_scale: scale,
                y_scale: scale,
                bg_color: WHITE,
            },
        );
        pixmap.into_png().ok()
    }
}

/// The eyes, asked to copy out one pictured page (a PNG).
pub type Sight<'s> = &'s mut dyn FnMut(&[u8]) -> Result<String, String>;

/// How a book was read: the text to hand, and whether the eyes read any of it.
pub struct Reading {
    pub text: String,
    pub seen: bool,
}

/// Reads every page: its own text where it has one, the eyes' copy (`see`) where not.
/// With no eyes, a picture page is noted as unread; a book of pictures only is refused.
pub fn read(book: &Book, mut see: Option<Sight<'_>>) -> Result<Reading, String> {
    let n = book.page_count();
    if n == 0 {
        return Err("this PDF has no pages".into());
    }
    let (mut text, mut seen, mut unread, mut seen_pages) = (String::new(), false, 0, 0);
    for i in 0..n {
        let body = match book.text_of(i) {
            Some(t) => format!("(its own text)\n{t}"),
            None => match see.as_mut() {
                Some(_) if seen_pages >= MAX_SEEN_PAGES => {
                    unread += 1;
                    format!("(not read: only the first {MAX_SEEN_PAGES} pictured pages are read)")
                }
                Some(see) => {
                    let png = book.picture_of(i).ok_or("a page could not be drawn")?;
                    let copy =
                        see(&png).map_err(|e| format!("page {} could not be seen: {e}", i + 1))?;
                    seen = true;
                    seen_pages += 1;
                    format!("(read by sight; it may hold mistakes)\n{}", copy.trim())
                }
                None => {
                    unread += 1;
                    "(not read: this page is a picture, and the spirits have no sight yet)".into()
                }
            },
        };
        text.push_str(&format!("--- page {} of {n} {body}\n\n", i + 1));
    }
    if unread == n {
        return Err(
            "this PDF holds only pictures (a scan): the spirits need sight to read it (see the README)"
                .into(),
        );
    }
    Ok(Reading {
        text: text.trim_end().to_string(),
        seen,
    })
}
