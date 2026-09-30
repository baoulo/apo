//! Enterprise-friendly static SVG badges (no external CDN).

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::report::{EvidenceReport, Report};

/// Color for a score band (shields-style).
pub fn score_color(score: Option<f64>) -> &'static str {
    match score {
        None => "#9f9f9f",
        Some(s) if s >= 80.0 => "#4c1",
        Some(s) if s >= 50.0 => "#dfb317",
        Some(_) => "#e05d44",
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Approximate text width for DejaVu Sans-like metrics (shields heuristic).
fn text_width(s: &str) -> f64 {
    s.chars()
        .map(|c| if c.is_ascii() { 6.5 } else { 9.0 })
        .sum()
}

fn segment_width(label: &str) -> f64 {
    (text_width(label) + 10.0).max(24.0)
}

/// Single flat badge: left label, right value.
pub fn flat_badge_svg(label: &str, message: &str, color: &str) -> String {
    let label_w = segment_width(label);
    let msg_w = segment_width(message);
    let total = label_w + msg_w;
    let label_cx = label_w / 2.0;
    let msg_cx = label_w + msg_w / 2.0;
    let label_esc = escape_xml(label);
    let msg_esc = escape_xml(message);
    let color_esc = escape_xml(color);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total:.0}" height="20" role="img" aria-label="{label_esc}: {msg_esc}">
  <title>{label_esc}: {msg_esc}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r"><rect width="{total:.0}" height="20" rx="3" fill="#fff"/></clipPath>
  <g clip-path="url(#r)">
    <rect width="{label_w:.0}" height="20" fill="#555"/>
    <rect x="{label_w:.0}" width="{msg_w:.0}" height="20" fill="{color_esc}"/>
    <rect width="{total:.0}" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110">
    <text aria-hidden="true" x="{label_cx:.1}" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="{label_tl:.0}">{label_esc}</text>
    <text x="{label_cx:.1}" y="140" transform="scale(.1)" textLength="{label_tl:.0}">{label_esc}</text>
    <text aria-hidden="true" x="{msg_cx:.1}" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="{msg_tl:.0}">{msg_esc}</text>
    <text x="{msg_cx:.1}" y="140" transform="scale(.1)" textLength="{msg_tl:.0}">{msg_esc}</text>
  </g>
</svg>
"##,
        total = total,
        label_w = label_w,
        msg_w = msg_w,
        label_cx = label_cx * 10.0,
        msg_cx = msg_cx * 10.0,
        label_tl = text_width(label) * 10.0,
        msg_tl = text_width(message) * 10.0,
        label_esc = label_esc,
        msg_esc = msg_esc,
        color_esc = color_esc,
    )
}

/// Hygiene badge SVG (`apo hygiene` · `{score}/100` or `n/a`).
pub fn hygiene_badge_svg(score: Option<f64>) -> String {
    let message = match score {
        Some(s) => format!("{s:.0}/100"),
        None => "n/a".into(),
    };
    flat_badge_svg("apo hygiene", &message, score_color(score))
}

/// Evidence badge: two flat badges side-by-side (knowledge + AI).
pub fn evidence_badge_svg(knowledge: f64, ai: f64) -> String {
    let k_msg = format!("{knowledge:.0}/100");
    let a_msg = format!("{ai:.0}/100");
    let k_color = score_color(Some(knowledge));
    let a_color = score_color(Some(ai));

    let k_label = "knowledge";
    let a_label = "ai";
    let k_lw = segment_width(k_label);
    let k_mw = segment_width(&k_msg);
    let a_lw = segment_width(a_label);
    let a_mw = segment_width(&a_msg);
    let gap = 4.0;
    let k_total = k_lw + k_mw;
    let a_total = a_lw + a_mw;
    let total = k_total + gap + a_total;

    let k_svg = flat_badge_svg(k_label, &k_msg, k_color);
    let a_svg = flat_badge_svg(a_label, &a_msg, a_color);

    // Strip outer svg wrappers and nest in a combined canvas.
    let k_inner = strip_outer_svg(&k_svg);
    let a_inner = strip_outer_svg(&a_svg);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total:.0}" height="20" role="img" aria-label="apo evidence: knowledge {k_msg}, ai {a_msg}">
  <title>apo evidence: knowledge {k_msg}, ai {a_msg}</title>
  <g>{k_inner}</g>
  <g transform="translate({offset:.0},0)">{a_inner}</g>
</svg>
"##,
        total = total,
        k_msg = escape_xml(&k_msg),
        a_msg = escape_xml(&a_msg),
        k_inner = k_inner,
        a_inner = a_inner,
        offset = k_total + gap,
    )
}

fn strip_outer_svg(svg: &str) -> String {
    let start = svg.find('>').map(|i| i + 1).unwrap_or(0);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    svg[start..end].trim().to_string()
}

impl Report {
    /// Default hygiene badge filename.
    pub fn badge_filename(&self) -> String {
        format!("{}-repository-hygiene-badge.svg", self.artifact_prefix())
    }
}

impl EvidenceReport {
    /// Default evidence badge filename.
    pub fn badge_filename(&self) -> String {
        format!("{}-repository-evidence-badge.svg", self.artifact_prefix())
    }
}

/// Resolve where to write a badge given `--badge-output` / `--output` and default dir.
pub fn resolve_badge_path(
    default_name: &str,
    badge_output: Option<&Path>,
    report_output: Option<&Path>,
    default_dir: &Path,
) -> PathBuf {
    if let Some(p) = badge_output {
        if p.is_dir() || p.extension().is_none() {
            return p.join(default_name);
        }
        return p.to_path_buf();
    }
    match report_output {
        None => default_dir.join(default_name),
        Some(p) if p.is_dir() || p.extension().is_none() => p.join(default_name),
        Some(p) => {
            let parent = p.parent().unwrap_or(default_dir);
            parent.join(default_name)
        }
    }
}

/// Write hygiene badge SVG for a report.
pub fn write_hygiene_badge(
    report: &Report,
    badge_output: Option<&Path>,
    report_output: Option<&Path>,
    default_dir: &Path,
) -> Result<PathBuf> {
    let path = resolve_badge_path(
        &report.badge_filename(),
        badge_output,
        report_output,
        default_dir,
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let svg = hygiene_badge_svg(report.policy.overall_score);
    std::fs::write(&path, svg)?;
    Ok(path)
}

/// Write evidence badge SVG for a report.
pub fn write_evidence_badge(
    report: &EvidenceReport,
    badge_output: Option<&Path>,
    report_output: Option<&Path>,
    default_dir: &Path,
) -> Result<PathBuf> {
    let path = resolve_badge_path(
        &report.badge_filename(),
        badge_output,
        report_output,
        default_dir,
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let svg = evidence_badge_svg(report.knowledge_maturity, report.ai_maturity);
    std::fs::write(&path, svg)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_bands() {
        assert_eq!(score_color(Some(90.0)), "#4c1");
        assert_eq!(score_color(Some(80.0)), "#4c1");
        assert_eq!(score_color(Some(50.0)), "#dfb317");
        assert_eq!(score_color(Some(49.9)), "#e05d44");
        assert_eq!(score_color(None), "#9f9f9f");
    }

    #[test]
    fn hygiene_svg_contains_label_and_score() {
        let svg = hygiene_badge_svg(Some(72.4));
        assert!(svg.contains("apo hygiene"));
        assert!(svg.contains("72/100"));
        assert!(svg.contains("#dfb317"));
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn hygiene_svg_na() {
        let svg = hygiene_badge_svg(None);
        assert!(svg.contains("n/a"));
        assert!(svg.contains("#9f9f9f"));
    }

    #[test]
    fn evidence_svg_dual_scores() {
        let svg = evidence_badge_svg(85.0, 40.0);
        assert!(svg.contains("knowledge"));
        assert!(svg.contains("85/100"));
        assert!(svg.contains("ai"));
        assert!(svg.contains("40/100"));
        assert!(svg.contains("#4c1"));
        assert!(svg.contains("#e05d44"));
    }
}
